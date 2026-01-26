use std::cmp;
use std::ffi::CString;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};

use libc::{IPC_CREAT, IPC_PRIVATE, SHM_R, SHM_W, shmat, shmctl, shmdt, shmget};
use x11::xfixes;
use x11::xlib;
use x11::xrender;
use x11::xshm;

// X11 errors are async; MIT-SHM failures (BadShmSeg) can otherwise kill the process.
// We trap X errors around XShmGetImage and fall back to XGetImage.
static X11_SHM_ERROR: AtomicBool = AtomicBool::new(false);
static CURSOR_CAPTURE_LOGGED: AtomicBool = AtomicBool::new(false);

unsafe extern "C" fn x11_error_handler(
    _display: *mut xlib::Display,
    _event: *mut xlib::XErrorEvent,
) -> libc::c_int {
    X11_SHM_ERROR.store(true, Ordering::SeqCst);
    0
}

#[derive(Debug)]
pub struct X11CaptureConfig {
    pub display: String,
}

#[derive(Debug)]
struct ShmState {
    image: *mut xlib::XImage,
    info: xshm::XShmSegmentInfo,
}

#[derive(Debug)]
pub struct X11Capture {
    display: *mut xlib::Display,
    root: xlib::Window,
    width: u32,
    height: u32,
    use_xshm: bool,
    shm: Option<ShmState>,
    cursor_capture: CursorCapture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CursorCapture {
    Disabled,
    XFixes,
}

pub fn init(config: X11CaptureConfig) -> Result<X11Capture, String> {
    if config.display.trim().is_empty() {
        return Err("X11 display cannot be empty".to_string());
    }

    let c_display = CString::new(config.display.clone())
        .map_err(|_| "X11 display contains an interior null byte".to_string())?;
    let display = unsafe { xlib::XOpenDisplay(c_display.as_ptr()) };
    if display.is_null() {
        return Err(format!("Failed to open X11 display {}", config.display));
    }

    let screen = unsafe { xlib::XDefaultScreen(display) };
    let root = unsafe { xlib::XRootWindow(display, screen) };
    let screen_width = unsafe { xlib::XDisplayWidth(display, screen) } as u32;
    let screen_height = unsafe { xlib::XDisplayHeight(display, screen) } as u32;

    let width = cmp::min(1920, screen_width);
    let height = cmp::min(1080, screen_height);

    let mut capture = X11Capture {
        display,
        root,
        width,
        height,
        use_xshm: false,
        shm: None,
        cursor_capture: CursorCapture::Disabled,
    };

    println!(
        "Configuring X11 capture for display {} at {}x{}",
        config.display, width, height
    );

    if unsafe { xshm::XShmQueryExtension(display) } != 0 {
        match capture.init_shm() {
            Ok(()) => {
                capture.use_xshm = true;
                println!("XShm extension detected; using shared-memory capture.");
            }
            Err(err) => {
                eprintln!("XShm initialization failed: {err}. Falling back to XGetImage.");
            }
        }
    } else {
        println!("XShm extension unavailable; using XGetImage fallback.");
    }

    let mut xfixes_event_base = 0;
    let mut xfixes_error_base = 0;
    let xfixes_supported = unsafe {
        xfixes::XFixesQueryExtension(display, &mut xfixes_event_base, &mut xfixes_error_base) != 0
    };

    let mut xrender_event_base = 0;
    let mut xrender_error_base = 0;
    let xrender_supported = unsafe {
        xrender::XRenderQueryExtension(display, &mut xrender_event_base, &mut xrender_error_base)
            != 0
    };

    if xfixes_supported {
        capture.cursor_capture = CursorCapture::XFixes;
        println!("XFixes cursor capture enabled; compositing cursor into frames.");
    } else if xrender_supported {
        println!(
            "XRender extension detected but XFixes is unavailable; cursor capture disabled."
        );
    } else {
        println!("No XFixes/XRender cursor support detected; cursor capture disabled.");
    }

    Ok(capture)
}

impl X11Capture {
    /// Capture a single frame from the primary display.
    ///
    /// Returns (pixel bytes, width, height) where pixels are in BGRA 8-bit format
    /// (native X11 32-bit pixel layout for common visuals).
    pub fn capture_frame(&mut self) -> Result<(Vec<u8>, u32, u32), String> {
        if self.display.is_null() {
            return Err("X11 display connection is not available".to_string());
        }

        let all_planes_raw = unsafe { xlib::XAllPlanes() };
        let all_planes_shm: u32 = all_planes_raw.try_into().unwrap_or(u32::MAX);
        let all_planes_get: u64 = all_planes_raw;

        if self.use_xshm {
            // If we ever get a BadShmSeg-style X error, disable XShm and fall back.
            let shm = match self.shm.as_ref() {
                Some(s) => s,
                None => {
                    self.use_xshm = false;
                    return self.capture_frame();
                }
            };

            X11_SHM_ERROR.store(false, Ordering::SeqCst);

            // Install temporary error handler to trap BadShmSeg (and similar) instead of aborting.
            let old_handler = unsafe { xlib::XSetErrorHandler(Some(x11_error_handler)) };

            // Flush any pending errors before our request so we attribute errors correctly.
            unsafe { xlib::XSync(self.display, xlib::False) };

            let status = unsafe {
                xshm::XShmGetImage(self.display, self.root, shm.image, 0, 0, all_planes_shm)
            };

            // Force server to process and deliver any error for the call above.
            unsafe { xlib::XSync(self.display, xlib::False) };

            // Restore prior handler.
            unsafe {
                xlib::XSetErrorHandler(old_handler);
            }

            if status == 0 || X11_SHM_ERROR.load(Ordering::SeqCst) {
                eprintln!(
                    "XShmGetImage failed (BadShmSeg or X error). Disabling XShm and falling back to XGetImage."
                );
                self.disable_xshm();
                return self.capture_frame(); // retry once using XGetImage
            }

            let mut result = unsafe { self.copy_image(shm.image) }?;
            self.composite_cursor_if_needed(&mut result);
            Ok(result)
        } else {
            self.capture_frame_xgetimage(all_planes_get)
        }
    }

    pub fn next_frame(&mut self) -> Result<(Vec<u8>, u32, u32), String> {
        self.capture_frame()
    }

    fn capture_frame_xgetimage(
        &mut self,
        all_planes_get: u64,
    ) -> Result<(Vec<u8>, u32, u32), String> {
        let image = unsafe {
            xlib::XGetImage(
                self.display,
                self.root,
                0,
                0,
                self.width,
                self.height,
                all_planes_get,
                xlib::ZPixmap,
            )
        };

        if image.is_null() {
            eprintln!("XGetImage returned null.");
            return Err("Failed to capture frame with XGetImage".to_string());
        }

        let mut result = unsafe { self.copy_image(image) }?;
        self.composite_cursor_if_needed(&mut result);

        unsafe {
            xlib::XDestroyImage(image);
        }

        Ok(result)
    }

    fn disable_xshm(&mut self) {
        self.use_xshm = false;

        if let Some(shm) = self.shm.take() {
            let mut info = shm.info;
            unsafe {
                // Best-effort cleanup; ignore failures.
                xshm::XShmDetach(self.display, &mut info);
                xlib::XDestroyImage(shm.image);
                shmdt(info.shmaddr as *mut _);
            }
        }
    }

    fn init_shm(&mut self) -> Result<(), String> {
        let screen = unsafe { xlib::XDefaultScreen(self.display) };
        let visual = unsafe { xlib::XDefaultVisual(self.display, screen) };
        let depth = unsafe { xlib::XDefaultDepth(self.display, screen) } as u32;

        let mut info: xshm::XShmSegmentInfo = unsafe { std::mem::zeroed() };

        let image = unsafe {
            xshm::XShmCreateImage(
                self.display,
                visual,
                depth,
                xlib::ZPixmap,
                ptr::null_mut(),
                &mut info,
                self.width,
                self.height,
            )
        };
        if image.is_null() {
            return Err("XShmCreateImage returned null".to_string());
        }

        let image_size = unsafe { (*image).bytes_per_line as usize * (*image).height as usize };

        let shmid = unsafe { shmget(IPC_PRIVATE, image_size, IPC_CREAT | SHM_R | SHM_W) };
        if shmid == -1 {
            unsafe { xlib::XDestroyImage(image) };
            return Err("shmget failed for XShm buffer".to_string());
        }
        info.shmid = shmid;

        let shmaddr = unsafe { shmat(shmid, ptr::null(), 0) };
        if shmaddr == (-1isize) as *mut _ {
            unsafe {
                xlib::XDestroyImage(image);
                shmctl(shmid, libc::IPC_RMID, ptr::null_mut());
            }
            return Err("shmat failed for XShm buffer".to_string());
        }

        info.shmaddr = shmaddr as *mut i8;
        info.readOnly = xlib::False;

        unsafe {
            (*image).data = info.shmaddr as *mut _;
        }

        let attach_status = unsafe { xshm::XShmAttach(self.display, &mut info) };
        if attach_status == 0 {
            unsafe {
                xlib::XDestroyImage(image);
                shmdt(info.shmaddr as *mut _);
                shmctl(shmid, libc::IPC_RMID, ptr::null_mut());
            }
            return Err("XShmAttach failed".to_string());
        }

        unsafe {
            xlib::XSync(self.display, xlib::False);
            // Mark for deletion; segment persists until last detach.
            shmctl(shmid, libc::IPC_RMID, ptr::null_mut());
        }

        self.shm = Some(ShmState { image, info });
        Ok(())
    }

    unsafe fn copy_image(&self, image: *mut xlib::XImage) -> Result<(Vec<u8>, u32, u32), String> {
        if image.is_null() {
            return Err("Cannot copy null XImage".to_string());
        }

        let (size, data_ptr) = unsafe {
            (
                (*image).bytes_per_line as usize * (*image).height as usize,
                (*image).data,
            )
        };

        if size == 0 || data_ptr.is_null() {
            return Err("Captured XImage has no data".to_string());
        }

        let buffer = unsafe { std::slice::from_raw_parts(data_ptr as *const u8, size) };
        Ok((buffer.to_vec(), self.width, self.height))
    }

    fn composite_cursor_if_needed(&self, frame: &mut (Vec<u8>, u32, u32)) {
        if self.cursor_capture != CursorCapture::XFixes {
            return;
        }

        let cursor = unsafe { xfixes::XFixesGetCursorImage(self.display) };
        if cursor.is_null() {
            return;
        }

        if CURSOR_CAPTURE_LOGGED
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            println!("Compositing XFixes cursor image into captured frames.");
        }

        unsafe {
            self.composite_cursor_image(cursor, frame);
            xlib::XFree(cursor as *mut _);
        }
    }

    unsafe fn composite_cursor_image(
        &self,
        cursor: *mut xfixes::XFixesCursorImage,
        frame: &mut (Vec<u8>, u32, u32),
    ) {
        if cursor.is_null() {
            return;
        }

        let (buffer, frame_width, frame_height) = frame;
        if buffer.is_empty() || *frame_width == 0 || *frame_height == 0 {
            return;
        }

        let cursor_ref = unsafe { &*cursor };
        let cursor_width = cursor_ref.width as i32;
        let cursor_height = cursor_ref.height as i32;
        if cursor_width <= 0 || cursor_height <= 0 {
            return;
        }

        let origin_x = i32::from(cursor_ref.x) - i32::from(cursor_ref.xhot);
        let origin_y = i32::from(cursor_ref.y) - i32::from(cursor_ref.yhot);

        let frame_width_i32 = *frame_width as i32;
        let frame_height_i32 = *frame_height as i32;

        for cy in 0..cursor_height {
            let dest_y = origin_y + cy;
            if dest_y < 0 || dest_y >= frame_height_i32 {
                continue;
            }

            for cx in 0..cursor_width {
                let dest_x = origin_x + cx;
                if dest_x < 0 || dest_x >= frame_width_i32 {
                    continue;
                }

                let cursor_index = (cy as usize * cursor_width as usize) + cx as usize;
                let pixel = unsafe { *cursor_ref.pixels.add(cursor_index) } as u64;
                let argb = (pixel & 0xFFFF_FFFF) as u32;

                let alpha = ((argb >> 24) & 0xFF) as u8;
                if alpha == 0 {
                    continue;
                }

                let src_r = ((argb >> 16) & 0xFF) as u8;
                let src_g = ((argb >> 8) & 0xFF) as u8;
                let src_b = (argb & 0xFF) as u8;

                let dest_index =
                    ((dest_y as u32 * *frame_width + dest_x as u32) * 4) as usize;

                if dest_index + 3 >= buffer.len() {
                    continue;
                }

                let dst_b = buffer[dest_index];
                let dst_g = buffer[dest_index + 1];
                let dst_r = buffer[dest_index + 2];

                let inv_alpha = 255u16 - alpha as u16;
                let alpha_u16 = alpha as u16;

                buffer[dest_index] =
                    ((src_b as u16 * alpha_u16 + dst_b as u16 * inv_alpha) / 255) as u8;
                buffer[dest_index + 1] =
                    ((src_g as u16 * alpha_u16 + dst_g as u16 * inv_alpha) / 255) as u8;
                buffer[dest_index + 2] =
                    ((src_r as u16 * alpha_u16 + dst_r as u16 * inv_alpha) / 255) as u8;
                buffer[dest_index + 3] = 0xFF;
            }
        }
    }
}

impl Drop for X11Capture {
    fn drop(&mut self) {
        // Ensure SHM is detached/destroyed if still enabled.
        self.disable_xshm();

        if !self.display.is_null() {
            unsafe { xlib::XCloseDisplay(self.display) };
            self.display = ptr::null_mut();
        }
    }
}
