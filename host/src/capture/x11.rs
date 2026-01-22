use std::cmp;
use std::ffi::CString;
use std::ptr;

use libc::{IPC_CREAT, IPC_PRIVATE, SHM_R, SHM_W, shmctl, shmdt, shmget, shmat};
use x11::xlib;
use x11::xshm;

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

    Ok(capture)
}

impl X11Capture {
    /// Capture a single frame from the primary display.
    ///
    /// Returns a tuple of (pixel bytes, width, height) where the pixels are in
    /// BGRA 8-bit format (native X11 32-bit pixel layout).
    pub fn capture_frame(&mut self) -> Result<(Vec<u8>, u32, u32), String> {
        if self.display.is_null() {
            return Err("X11 display connection is not available".to_string());
        }

        if self.use_xshm {
            let shm = self
                .shm
                .as_ref()
                .ok_or_else(|| "XShm state missing while XShm is enabled".to_string())?;
            let status = unsafe {
                xshm::XShmGetImage(
                    self.display,
                    self.root,
                    shm.image,
                    0,
                    0,
                    xlib::AllPlanes,
                )
            };
            if status == 0 {
                eprintln!("XShmGetImage failed to capture a frame.");
                return Err("Failed to capture frame with XShm".to_string());
            }
            unsafe { self.copy_image(shm.image) }
        } else {
            let image = unsafe {
                xlib::XGetImage(
                    self.display,
                    self.root,
                    0,
                    0,
                    self.width,
                    self.height,
                    xlib::AllPlanes,
                    xlib::ZPixmap,
                )
            };
            if image.is_null() {
                eprintln!("XGetImage returned null.");
                return Err("Failed to capture frame with XGetImage".to_string());
            }
            let result = unsafe { self.copy_image(image) };
            unsafe {
                xlib::XDestroyImage(image);
            }
            result
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
        let shmid = unsafe {
            shmget(
                IPC_PRIVATE,
                image_size,
                IPC_CREAT | SHM_R | SHM_W,
            )
        };
        if shmid == -1 {
            unsafe {
                xlib::XDestroyImage(image);
            }
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
        }

        unsafe {
            shmctl(shmid, libc::IPC_RMID, ptr::null_mut());
        }

        self.shm = Some(ShmState { image, info });
        Ok(())
    }

    unsafe fn copy_image(&self, image: *mut xlib::XImage) -> Result<(Vec<u8>, u32, u32), String> {
        if image.is_null() {
            return Err("Cannot copy null XImage".to_string());
        }
        let size = (*image).bytes_per_line as usize * (*image).height as usize;
        if size == 0 || (*image).data.is_null() {
            return Err("Captured XImage has no data".to_string());
        }
        let buffer = std::slice::from_raw_parts((*image).data as *const u8, size);
        Ok((buffer.to_vec(), self.width, self.height))
    }
}

impl Drop for X11Capture {
    fn drop(&mut self) {
        if let Some(shm) = self.shm.take() {
            let mut info = shm.info;
            unsafe {
                xshm::XShmDetach(self.display, &mut info);
                xlib::XDestroyImage(shm.image);
                shmdt(info.shmaddr as *mut _);
            }
        }
        if !self.display.is_null() {
            unsafe {
                xlib::XCloseDisplay(self.display);
            }
            self.display = ptr::null_mut();
        }
    }
}
