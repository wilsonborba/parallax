use std::env;

#[derive(Debug, Clone)]
pub struct CliConfig {
    pub display: String,
    pub bind_addr: String,
    pub target_addr: String,
    pub control_bind: String,
    pub pairing_token: String,
    pub prefer_vaapi: bool,
}

#[derive(Debug, Clone)]
pub struct VirtualDisplayConfig {
    pub id: String,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone)]
pub enum CliAction {
    Run(CliConfig),
    ListDisplays,
    VirtualBackendStatus,
    ListVirtualDisplays,
    EnableVirtualDisplay(VirtualDisplayConfig),
    DisableVirtualDisplay { id: String },
}

impl CliConfig {
    pub fn from_env() -> Result<CliAction, String> {
        let mut display = String::from(":0");
        let mut bind_addr = String::from("0.0.0.0:5000");
        let mut target_addr = String::from("127.0.0.1:5000");
        let mut control_bind = String::from("0.0.0.0:0");
        let mut pairing_token = String::from("auto");
        let mut prefer_vaapi = true;

        let mut list_displays = false;
        let mut virtual_backend_status = false;
        let mut list_virtual_displays = false;
        let mut enable_virtual_id: Option<String> = None;
        let mut disable_virtual_id: Option<String> = None;
        let mut virtual_width: Option<u32> = None;
        let mut virtual_height: Option<u32> = None;
        let mut virtual_x: i32 = 0;
        let mut virtual_y: i32 = 0;

        let mut args = env::args().skip(1);

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--display" => {
                    display = args.next().ok_or("--display requires a value")?;
                }
                "--bind" => {
                    bind_addr = args.next().ok_or("--bind requires a value")?;
                }
                "--target" => {
                    target_addr = args.next().ok_or("--target requires a value")?;
                }
                "--control-bind" => {
                    control_bind = args.next().ok_or("--control-bind requires a value")?;
                }
                "--pairing-token" => {
                    pairing_token = args.next().ok_or("--pairing-token requires a value")?;
                }
                "--prefer-vaapi" => {
                    prefer_vaapi = true;
                }
                "--software" => {
                    prefer_vaapi = false;
                }
                "--list-displays" => {
                    list_displays = true;
                }
                "--virtual-backend-status" => {
                    virtual_backend_status = true;
                }
                "--list-virtual-displays" => {
                    list_virtual_displays = true;
                }
                "--enable-virtual-display" => {
                    enable_virtual_id = Some(
                        args.next()
                            .ok_or("--enable-virtual-display requires an id")?,
                    );
                }
                "--disable-virtual-display" => {
                    disable_virtual_id = Some(
                        args.next()
                            .ok_or("--disable-virtual-display requires an id")?,
                    );
                }
                "--virtual-width" => {
                    virtual_width = Some(
                        args.next()
                            .ok_or("--virtual-width requires a value")?
                            .parse::<u32>()
                            .map_err(|_| "--virtual-width expects an integer")?,
                    );
                }
                "--virtual-height" => {
                    virtual_height = Some(
                        args.next()
                            .ok_or("--virtual-height requires a value")?
                            .parse::<u32>()
                            .map_err(|_| "--virtual-height expects an integer")?,
                    );
                }
                "--virtual-x" => {
                    virtual_x = args
                        .next()
                        .ok_or("--virtual-x requires a value")?
                        .parse::<i32>()
                        .map_err(|_| "--virtual-x expects an integer")?;
                }
                "--virtual-y" => {
                    virtual_y = args
                        .next()
                        .ok_or("--virtual-y requires a value")?
                        .parse::<i32>()
                        .map_err(|_| "--virtual-y expects an integer")?;
                }
                "-h" | "--help" => {
                    return Err(Self::help());
                }
                other => {
                    return Err(format!("Unknown argument: {other}\n\n{}", Self::help()));
                }
            }
        }

        if let Some(id) = disable_virtual_id {
            return Ok(CliAction::DisableVirtualDisplay { id });
        }

        if let Some(id) = enable_virtual_id {
            let width = virtual_width.ok_or("--virtual-width is required")?;
            let height = virtual_height.ok_or("--virtual-height is required")?;
            return Ok(CliAction::EnableVirtualDisplay(VirtualDisplayConfig {
                id,
                width,
                height,
                x: virtual_x,
                y: virtual_y,
            }));
        }

        if list_virtual_displays {
            return Ok(CliAction::ListVirtualDisplays);
        }

        if virtual_backend_status {
            return Ok(CliAction::VirtualBackendStatus);
        }

        if list_displays {
            return Ok(CliAction::ListDisplays);
        }

        Ok(CliAction::Run(Self {
            display,
            bind_addr,
            target_addr,
            control_bind,
            pairing_token,
            prefer_vaapi,
        }))
    }

    pub fn help() -> String {
        [
            "Usage: prlx-hostd [options]",
            "",
            "Run options:",
            "  --display <DISPLAY>   X11 display to capture (default :0)",
            "  --bind <ADDR>         UDP bind address (default 0.0.0.0:5000)",
            "  --target <ADDR>       UDP target address (default 127.0.0.1:5000)",
            "  --control-bind <ADDR> TCP control bind address (default 0.0.0.0:0)",
            "  --pairing-token <KEY> Pairing token for control sessions (default auto)",
            "  --prefer-vaapi        Prefer VAAPI H.264 encoder (default)",
            "  --software            Force software H.264 encoder",
            "",
            "Display management:",
            "  --list-displays                    List available host displays and exit",
            "  --virtual-backend-status           Show virtual display backend diagnostics",
            "  --list-virtual-displays            List persisted virtual displays",
            "  --enable-virtual-display <ID>      Create/update virtual display",
            "    --virtual-width <PX>             Required with --enable-virtual-display",
            "    --virtual-height <PX>            Required with --enable-virtual-display",
            "    --virtual-x <PX>                 Optional (default 0)",
            "    --virtual-y <PX>                 Optional (default 0)",
            "  --disable-virtual-display <ID>     Remove virtual display",
            "",
            "General:",
            "  -h, --help            Print this help text",
        ]
        .join("\n")
    }
}
