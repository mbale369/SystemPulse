//! Fixed window actions only. No shell, arbitrary path, URL or window label from IPC.
use serde::Serialize;
use tauri::{Manager, PhysicalPosition, PhysicalSize, WebviewWindow};

pub fn lock_instance(directory: &std::path::Path) -> std::io::Result<std::fs::File> {
    let path = directory.join("systempulse-instance.lock");
    if let Ok(meta) = std::fs::symlink_metadata(&path)
        && (meta.file_type().is_symlink() || !meta.is_file())
    {
        return Err(std::io::Error::other("Verrou non sûr"));
    }
    let mut options = std::fs::OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options.open(&path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        let meta = file.metadata()?;
        let named = std::fs::symlink_metadata(&path)?;
        if named.file_type().is_symlink()
            || meta.ino() != named.ino()
            || meta.dev() != named.dev()
            || meta.uid() != directory.metadata()?.uid()
            || meta.nlink() != 1
            || meta.permissions().mode() & 0o077 != 0
        {
            return Err(std::io::Error::other("Verrou non sûr"));
        }
    }
    file.try_lock()
        .map_err(|_| std::io::Error::other("SystemPulse est déjà ouvert ; utilisez la capsule."))?;
    Ok(file)
}

pub fn authorize(label: &str, widget_only: bool) -> Result<(), String> {
    if label == "widget" || (!widget_only && label == "main") {
        Ok(())
    } else {
        Err("Fenêtre non autorisée".into())
    }
}

pub fn geometry(
    area: (i32, i32, u32, u32),
    scale: f64,
    expanded: bool,
    anchor: Option<(i32, i32, u32, u32)>,
) -> (i32, i32, u32, u32) {
    let scale = if scale.is_finite() {
        scale.clamp(1.0, 4.0)
    } else {
        1.0
    };
    let margin = (12.0 * scale) as i64;
    let (ax, ay, aw, ah) = area;
    let w = ((if expanded { 400.0 } else { 320.0 }) * scale) as i64;
    let h = ((if expanded { 620.0 } else { 112.0 }) * scale) as i64;
    let w = w.min(i64::from(aw).saturating_sub(margin * 2).max(1));
    let h = h.min(i64::from(ah).saturating_sub(margin * 2).max(1));
    let right = anchor.map_or(
        i64::from(ax) + i64::from(aw) - margin,
        |(x, _, width, _)| i64::from(x) + i64::from(width),
    );
    let bottom = anchor.map_or(
        i64::from(ay) + i64::from(ah) - margin,
        |(_, y, _, height)| i64::from(y) + i64::from(height),
    );
    let max_x = (i64::from(ax) + i64::from(aw) - w).max(i64::from(ax));
    let max_y = (i64::from(ay) + i64::from(ah) - h).max(i64::from(ay));
    (
        (right - w).clamp(i64::from(ax), max_x) as i32,
        (bottom - h).clamp(i64::from(ay), max_y) as i32,
        w as u32,
        h as u32,
    )
}

pub fn layout(window: &WebviewWindow, expanded: bool, preserve_anchor: bool) -> Result<(), String> {
    window
        .set_min_size(Some(tauri::LogicalSize::new(280.0, 96.0)))
        .map_err(|_| "Dimensions minimales impossibles")?;
    let monitor = window
        .current_monitor()
        .map_err(|_| "Écran indisponible")?
        .or(window.primary_monitor().map_err(|_| "Écran indisponible")?);
    if let Some(monitor) = monitor {
        let area = monitor.work_area();
        let anchor = if preserve_anchor {
            window
                .outer_position()
                .ok()
                .zip(window.outer_size().ok())
                .map(|(p, s)| (p.x, p.y, s.width, s.height))
        } else {
            None
        };
        let (x, y, w, h) = geometry(
            (
                area.position.x,
                area.position.y,
                area.size.width,
                area.size.height,
            ),
            monitor.scale_factor(),
            expanded,
            anchor,
        );
        window
            .set_size(PhysicalSize::new(w, h))
            .map_err(|_| "Dimensions impossibles")?;
        // Wayland may refuse global placement. Native drag and layout still work.
        let _ = window.set_position(PhysicalPosition::new(x, y));
    }
    Ok(())
}

#[tauri::command]
pub fn show_widget(app: tauri::AppHandle, window: WebviewWindow) -> Result<(), String> {
    authorize(window.label(), false)?;
    let widget = app
        .get_webview_window("widget")
        .ok_or("Capsule indisponible")?;
    widget.show().map_err(|_| "Affichage impossible".into())
}
#[tauri::command]
pub fn widget_expand(expanded: bool, window: WebviewWindow) -> Result<(), String> {
    authorize(window.label(), true)?;
    layout(&window, expanded, true)
}
#[tauri::command]
pub fn widget_drag(window: WebviewWindow) -> Result<(), String> {
    authorize(window.label(), true)?;
    window
        .start_dragging()
        .map_err(|_| "Déplacement non pris en charge par le bureau".into())
}
#[tauri::command]
pub fn widget_action(
    action: String,
    app: tauri::AppHandle,
    window: WebviewWindow,
) -> Result<(), String> {
    authorize(window.label(), true)?;
    match action.as_str() {
        "dashboard" | "hide" => {
            let main = app
                .get_webview_window("main")
                .ok_or("Fenêtre principale indisponible")?;
            main.show().map_err(|_| "Affichage impossible")?;
            main.set_focus().map_err(|_| "Focus impossible")?;
            if action == "hide" {
                window.hide().map_err(|_| "Masquage impossible")?;
            }
            Ok(())
        }
        "quit" => {
            app.exit(0);
            Ok(())
        }
        _ => Err("Action non autorisée".into()),
    }
}
#[derive(Serialize)]
pub struct McpLaunchInfo {
    command: String,
    available: bool,
}
#[tauri::command]
pub fn get_mcp_launch_info(window: WebviewWindow) -> Result<McpLaunchInfo, String> {
    authorize(window.label(), false)?;
    let exe = std::env::current_exe().map_err(|_| "Chemin d'application indisponible")?;
    let path = exe
        .parent()
        .ok_or("Installation indisponible")?
        .join(if cfg!(windows) {
            "systempulse-mcp.exe"
        } else {
            "systempulse-mcp"
        });
    Ok(McpLaunchInfo {
        available: path.is_file(),
        command: path.to_string_lossy().into_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn singleton_lock_is_exclusive_and_released_on_exit() {
        let dir = std::env::temp_dir().join(format!(
            "systempulse-lock-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir(&dir).expect("fixture");
        let first = lock_instance(&dir).expect("first");
        assert!(lock_instance(&dir).is_err());
        drop(first);
        assert!(lock_instance(&dir).is_ok());
        #[cfg(unix)]
        {
            let other = dir.join("linked");
            std::fs::create_dir(&other).expect("dir");
            std::os::unix::fs::symlink(
                dir.join("systempulse-instance.lock"),
                other.join("systempulse-instance.lock"),
            )
            .expect("link");
            assert!(lock_instance(&other).is_err());
        }
    }
    #[test]
    fn ipc_authority_is_bound_to_the_calling_window() {
        assert!(authorize("main", false).is_ok());
        assert!(authorize("widget", true).is_ok());
        for label in ["main", "remote", "widget/../main", ""] {
            assert!(authorize(label, true).is_err());
        }
        assert!(authorize("remote", false).is_err());
    }
    #[test]
    fn geometry_handles_dpi_negative_monitors_and_expansion() {
        for area in [
            (0, 0, 1920, 1080),
            (-1920, 0, 1920, 1080),
            (0, -1080, 800, 600),
            (0, 0, 240, 240),
        ] {
            for scale in [1.0, 1.5, 2.0, f64::NAN] {
                for expanded in [false, true] {
                    let (x, y, w, h) =
                        geometry(area, scale, expanded, Some((99999, -99999, 320, 112)));
                    assert!(x >= area.0 && y >= area.1);
                    assert!(i64::from(x) + i64::from(w) <= i64::from(area.0) + i64::from(area.2));
                    assert!(i64::from(y) + i64::from(h) <= i64::from(area.1) + i64::from(area.3));
                }
            }
        }
        let a = geometry((0, 0, 1920, 1080), 1.0, false, None);
        let b = geometry((0, 0, 1920, 1080), 1.0, true, Some(a));
        assert_eq!(a.0 + a.2 as i32, b.0 + b.2 as i32);
        assert_eq!(a.1 + a.3 as i32, b.1 + b.3 as i32);
    }
}
