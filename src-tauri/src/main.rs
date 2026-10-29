#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager, Window};

#[cfg(target_os = "windows")]
#[link(name = "user32")]
extern "system" {
    fn SetWindowDisplayAffinity(hwnd: isize, dw_affinity: u32) -> i32;
}

#[tauri::command]
fn set_stealth_mode(window: Window, enabled: bool) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        if let Ok(hwnd) = window.hwnd() {
            let flag = if enabled {
                0x00000011 // WDA_EXCLUDEFROMCAPTURE
            } else {
                0x00000000 // WDA_NONE
            };
            let ret = unsafe { SetWindowDisplayAffinity(hwnd.0 as isize, flag) };
            if ret == 0 {
                return Err("SetWindowDisplayAffinity failed".into());
            }
        }
    }
    Ok(enabled)
}

#[tauri::command]
fn set_always_on_top(window: Window, enabled: bool) -> Result<bool, String> {
    window
        .set_always_on_top(enabled)
        .map_err(|e| e.to_string())?;
    Ok(enabled)
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            if let Some(window) = app.get_window("main") {
                #[cfg(target_os = "windows")]
                {
                    if let Ok(hwnd) = window.hwnd() {
                        // Aktifkan WDA_EXCLUDEFROMCAPTURE saat aplikasi dibuka
                        // Di layar kamu tetap 100% kelihatan, tapi di SS / OBS / Record tembus pandang!
                        unsafe {
                            SetWindowDisplayAffinity(hwnd.0 as isize, 0x00000011);
                        }
                    }
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            set_stealth_mode,
            set_always_on_top
        ])
        .run(tauri::generate_context!())
        .expect("error while running Spectre application");
}
