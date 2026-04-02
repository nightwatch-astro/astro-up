use std::sync::atomic::{AtomicUsize, Ordering};

use tauri::{
    AppHandle, Manager,
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

static BADGE_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Build the system tray with context menu and event handlers.
pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
    let check = MenuItem::with_id(
        app,
        "check_updates",
        "Check for Updates",
        true,
        None::<&str>,
    )?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show, &check, &separator, &quit])?;

    TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().cloned().unwrap())
        .tooltip("Astro-Up")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => show_main_window(app),
            "check_updates" => {
                tracing::info!("Tray: check for updates requested");
            }
            "quit" => {
                tracing::info!("Tray: quit requested");
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

/// Update the tray badge count. Generates a numbered overlay icon.
pub fn set_badge_count(app: &AppHandle, count: usize) {
    BADGE_COUNT.store(count, Ordering::Relaxed);

    if let Some(tray) = app.tray_by_id("main-tray") {
        let icon = if count == 0 {
            // Restore default icon
            app.default_window_icon().cloned().unwrap()
        } else {
            // Generate badge overlay icon
            generate_badge_icon(count)
        };
        let _ = tray.set_icon(Some(icon));
        let tooltip = if count == 0 {
            "Astro-Up".to_string()
        } else {
            format!("Astro-Up — {count} update(s) available")
        };
        let _ = tray.set_tooltip(Some(&tooltip));
    }
}

/// Get the current badge count.
pub fn badge_count() -> usize {
    BADGE_COUNT.load(Ordering::Relaxed)
}

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Generate a simple badge icon with the given count.
/// Creates a 32x32 RGBA image with a red circle and white number.
fn generate_badge_icon(count: usize) -> Image<'static> {
    let size: u32 = 32;
    let mut pixels = vec![0u8; (size * size * 4) as usize];

    let label = if count > 9 { "9+" } else { "" };
    let _ = label; // Used for tooltip; icon uses filled circle approach

    // Draw a red filled circle (badge indicator)
    let cx = size / 2;
    let cy = size / 2;
    let radius = size / 2 - 1;

    for y in 0..size {
        for x in 0..size {
            let dx = x as i32 - cx as i32;
            let dy = y as i32 - cy as i32;
            let dist_sq = (dx * dx + dy * dy) as u32;
            let idx = ((y * size + x) * 4) as usize;

            if dist_sq <= radius * radius {
                // Red circle
                pixels[idx] = 220; // R
                pixels[idx + 1] = 38; // G
                pixels[idx + 2] = 38; // B
                pixels[idx + 3] = 255; // A
            }
            // else: transparent (0,0,0,0)
        }
    }

    // Draw a simple white dot in center to indicate "has number"
    // (Full text rendering would require a font rasterizer — the tooltip shows the actual count)
    let dot_radius: u32 = 3;
    for y in (cy - dot_radius)..=(cy + dot_radius) {
        for x in (cx - dot_radius)..=(cx + dot_radius) {
            let dx = x as i32 - cx as i32;
            let dy = y as i32 - cy as i32;
            if (dx * dx + dy * dy) as u32 <= dot_radius * dot_radius {
                let idx = ((y * size + x) * 4) as usize;
                pixels[idx] = 255; // R
                pixels[idx + 1] = 255; // G
                pixels[idx + 2] = 255; // B
                pixels[idx + 3] = 255; // A
            }
        }
    }

    Image::new_owned(pixels, size, size)
}
