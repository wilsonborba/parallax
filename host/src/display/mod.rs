use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct DisplayInfo {
    pub id: String,
    pub name: String,
    pub primary: bool,
    pub connected: bool,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub pos_x: Option<i32>,
    pub pos_y: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct VirtualDisplay {
    pub id: String,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct VirtualBackendStatus {
    pub session_type: String,
    pub display_env: Option<String>,
    pub wayland_display_env: Option<String>,
    pub xrandr_available: bool,
    pub xrandr_query_ok: bool,
    pub xrandr_setmonitor_supported: bool,
    pub vkms_loaded: bool,
}

pub fn list_displays() -> Result<Vec<DisplayInfo>, String> {
    match Command::new("xrandr").arg("--query").output() {
        Ok(output) => {
            if !output.status.success() {
                return Ok(fallback_display());
            }

            let text = String::from_utf8_lossy(&output.stdout);
            let mut displays = Vec::new();

            for line in text.lines() {
                if !line.contains(" connected") {
                    continue;
                }

                let tokens: Vec<&str> = line.split_whitespace().collect();
                if tokens.is_empty() {
                    continue;
                }

                let name = tokens[0].to_string();
                let primary = tokens.contains(&"primary");

                let mut width = None;
                let mut height = None;
                let mut pos_x = None;
                let mut pos_y = None;

                for token in &tokens {
                    if let Some((w, h, x, y)) = parse_geometry(token) {
                        width = Some(w);
                        height = Some(h);
                        pos_x = Some(x);
                        pos_y = Some(y);
                        break;
                    }
                }

                displays.push(DisplayInfo {
                    id: name.clone(),
                    name,
                    primary,
                    connected: true,
                    width,
                    height,
                    pos_x,
                    pos_y,
                });
            }

            if displays.is_empty() {
                Ok(fallback_display())
            } else {
                Ok(displays)
            }
        }
        Err(err) => {
            if err.kind() == std::io::ErrorKind::NotFound {
                Ok(fallback_display())
            } else {
                Err(format!("Failed to run xrandr: {err}"))
            }
        }
    }
}

pub fn virtual_backend_status() -> VirtualBackendStatus {
    let session_type = env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "unknown".to_string());
    let display_env = env::var("DISPLAY").ok();
    let wayland_display_env = env::var("WAYLAND_DISPLAY").ok();

    let mut xrandr_available = false;
    let mut xrandr_query_ok = false;
    let mut xrandr_setmonitor_supported = false;

    if let Ok(output) = Command::new("xrandr").arg("--help").output() {
        xrandr_available = output.status.success();
        if xrandr_available {
            let help = String::from_utf8_lossy(&output.stdout);
            xrandr_setmonitor_supported = help.contains("--setmonitor");
        }
    }

    if xrandr_available {
        if let Ok(output) = Command::new("xrandr").arg("--query").output() {
            xrandr_query_ok = output.status.success();
        }
    }

    let vkms_loaded = PathBuf::from("/sys/module/vkms").exists();

    VirtualBackendStatus {
        session_type,
        display_env,
        wayland_display_env,
        xrandr_available,
        xrandr_query_ok,
        xrandr_setmonitor_supported,
        vkms_loaded,
    }
}

pub fn list_virtual_displays() -> Result<Vec<VirtualDisplay>, String> {
    let path = virtual_config_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&path)
        .map_err(|err| format!("Failed to read {}: {err}", path.display()))?;
    let mut displays = Vec::new();

    for (idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = trimmed.split(',').collect();
        if parts.len() != 6 {
            return Err(format!(
                "Invalid virtual display config at line {}",
                idx + 1
            ));
        }

        let id = parts[0].to_string();
        let width = parts[1]
            .parse::<u32>()
            .map_err(|_| format!("Invalid width at line {}", idx + 1))?;
        let height = parts[2]
            .parse::<u32>()
            .map_err(|_| format!("Invalid height at line {}", idx + 1))?;
        let x = parts[3]
            .parse::<i32>()
            .map_err(|_| format!("Invalid x at line {}", idx + 1))?;
        let y = parts[4]
            .parse::<i32>()
            .map_err(|_| format!("Invalid y at line {}", idx + 1))?;
        let enabled = matches!(parts[5], "1" | "true" | "yes");

        displays.push(VirtualDisplay {
            id,
            width,
            height,
            x,
            y,
            enabled,
        });
    }

    Ok(displays)
}

pub fn enable_virtual_display(display: VirtualDisplay) -> Result<(), String> {
    apply_setmonitor(&display)?;
    upsert_virtual_display(&display)?;
    Ok(())
}

pub fn apply_persisted_virtual_displays() -> Result<Vec<String>, String> {
    let displays = list_virtual_displays()?;
    let mut applied = Vec::new();

    for d in displays {
        if !d.enabled {
            continue;
        }
        apply_setmonitor(&d)?;
        applied.push(d.id);
    }

    Ok(applied)
}

pub fn disable_virtual_display(id: &str) -> Result<(), String> {
    let status = Command::new("xrandr").arg("--delmonitor").arg(id).status();
    if let Ok(exit) = status {
        if !exit.success() {
            return Err(format!("xrandr --delmonitor failed for {id}"));
        }
    }

    let mut all = list_virtual_displays()?;
    all.retain(|d| d.id != id);
    write_virtual_displays(&all)?;
    Ok(())
}

pub fn format_displays(displays: &[DisplayInfo]) -> String {
    let mut out = String::from("Detected displays:\n");
    out.push_str("id\tname\tprimary\tconnected\tgeometry\n");

    for d in displays {
        let geometry = match (d.width, d.height, d.pos_x, d.pos_y) {
            (Some(w), Some(h), Some(x), Some(y)) => format!("{w}x{h}+{x}+{y}"),
            _ => "unknown".to_string(),
        };

        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\n",
            d.id, d.name, d.primary, d.connected, geometry
        ));
    }

    out
}

pub fn format_virtual_displays(displays: &[VirtualDisplay]) -> String {
    if displays.is_empty() {
        return "No virtual displays configured.\n".to_string();
    }

    let mut out = String::from("Configured virtual displays:\n");
    out.push_str("id\tenabled\tgeometry\n");

    for d in displays {
        out.push_str(&format!(
            "{}\t{}\t{}x{}+{}+{}\n",
            d.id, d.enabled, d.width, d.height, d.x, d.y
        ));
    }

    out
}

pub fn format_virtual_backend_status(status: &VirtualBackendStatus) -> String {
    let mut out = String::from("Virtual display backend status:\n");
    out.push_str(&format!("session_type={}\n", status.session_type));
    out.push_str(&format!(
        "DISPLAY={}\n",
        status.display_env.as_deref().unwrap_or("<unset>")
    ));
    out.push_str(&format!(
        "WAYLAND_DISPLAY={}\n",
        status.wayland_display_env.as_deref().unwrap_or("<unset>")
    ));
    out.push_str(&format!("xrandr_available={}\n", status.xrandr_available));
    out.push_str(&format!("xrandr_query_ok={}\n", status.xrandr_query_ok));
    out.push_str(&format!(
        "xrandr_setmonitor_supported={}\n",
        status.xrandr_setmonitor_supported
    ));
    out.push_str(&format!("vkms_loaded={}\n", status.vkms_loaded));

    out.push('\n');
    if status.session_type.eq_ignore_ascii_case("wayland") {
        out.push_str("note=Wayland session detected; xrandr virtual monitors may be unavailable.\n");
    } else if !status.xrandr_available {
        out.push_str("note=xrandr not found; install x11-xserver-utils.\n");
    } else if !status.xrandr_setmonitor_supported {
        out.push_str("note=this xrandr build does not support --setmonitor.\n");
    }

    out
}

fn apply_setmonitor(display: &VirtualDisplay) -> Result<(), String> {
    let backend = virtual_backend_status();
    if !backend.xrandr_available {
        return Err("xrandr is not available. Install x11-xserver-utils.".to_string());
    }
    if !backend.xrandr_setmonitor_supported {
        return Err("xrandr does not support --setmonitor on this system.".to_string());
    }
    if backend.session_type.eq_ignore_ascii_case("wayland") {
        return Err(
            "Wayland session detected; virtual monitors via xrandr usually require X11 session."
                .to_string(),
        );
    }

    // If a monitor with the same id already exists, delete it first so updates are idempotent.
    let _ = Command::new("xrandr")
        .arg("--delmonitor")
        .arg(&display.id)
        .status();

    let geometry = format!(
        "{w}/{w}x{h}/{h}+{x}+{y}",
        w = display.width,
        h = display.height,
        x = display.x,
        y = display.y
    );

    let output = Command::new("xrandr")
        .arg("--setmonitor")
        .arg(&display.id)
        .arg(geometry)
        .arg("none")
        .output()
        .map_err(|err| format!("Failed to run xrandr --setmonitor: {err}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(
            if stderr.is_empty() {
                "xrandr --setmonitor failed. Check DISPLAY/X11 session and monitor naming conflicts."
                    .to_string()
            } else {
                format!(
                    "xrandr --setmonitor failed: {stderr}. Check DISPLAY/X11 session and monitor naming conflicts."
                )
            },
        );
    }

    Ok(())
}

fn upsert_virtual_display(display: &VirtualDisplay) -> Result<(), String> {
    let mut all = list_virtual_displays()?;
    all.retain(|d| d.id != display.id);

    let mut saved = display.clone();
    saved.enabled = true;
    all.push(saved);

    write_virtual_displays(&all)
}

fn write_virtual_displays(displays: &[VirtualDisplay]) -> Result<(), String> {
    let path = virtual_config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("Failed to create {}: {err}", parent.display()))?;
    }

    let mut content = String::from("# id,width,height,x,y,enabled\n");
    for d in displays {
        let enabled = if d.enabled { "1" } else { "0" };
        content.push_str(&format!(
            "{},{},{},{},{},{}\n",
            d.id, d.width, d.height, d.x, d.y, enabled
        ));
    }

    fs::write(&path, content).map_err(|err| format!("Failed to write {}: {err}", path.display()))
}

fn virtual_config_path() -> Result<PathBuf, String> {
    let home = env::var("HOME").map_err(|_| "HOME is not set".to_string())?;
    Ok(PathBuf::from(home).join(".config/parallax/virtual_displays.conf"))
}

fn parse_geometry(token: &str) -> Option<(u32, u32, i32, i32)> {
    let (w_str, rest) = token.split_once('x')?;
    let (h_str, rest) = rest.split_once('+')?;
    let (x_str, y_str) = rest.split_once('+')?;

    let width = w_str.parse::<u32>().ok()?;
    let height = h_str.parse::<u32>().ok()?;
    let pos_x = x_str.parse::<i32>().ok()?;
    let pos_y = y_str.parse::<i32>().ok()?;

    Some((width, height, pos_x, pos_y))
}

fn fallback_display() -> Vec<DisplayInfo> {
    let name = env::var("DISPLAY").unwrap_or_else(|_| ":0".to_string());
    vec![DisplayInfo {
        id: name.clone(),
        name,
        primary: true,
        connected: true,
        width: None,
        height: None,
        pos_x: None,
        pos_y: None,
    }]
}
