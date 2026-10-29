#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager, Window};

#[cfg(target_os = "windows")]
#[link(name = "user32")]
extern "system" {
    fn SetWindowDisplayAffinity(hwnd: isize, dw_affinity: u32) -> i32;
    fn EnumChildWindows(
        hwnd_parent: isize,
        lp_enum_func: unsafe extern "system" fn(hwnd: isize, lparam: isize) -> i32,
        lparam: isize,
    ) -> i32;
    fn InvalidateRect(hwnd: isize, lp_rect: *const std::ffi::c_void, b_erase: i32) -> i32;
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn enum_child_affinity(hwnd: isize, lparam: isize) -> i32 {
    let flag = lparam as u32;
    SetWindowDisplayAffinity(hwnd, flag);
    InvalidateRect(hwnd, std::ptr::null(), 1);
    1 // Lanjut ke semua child HWND berikutnya
}

#[cfg(target_os = "windows")]
fn apply_stealth_recursive(hwnd: isize, enabled: bool) {
    let flag = if enabled {
        0x00000011 // WDA_EXCLUDEFROMCAPTURE: Gaib di SS / OBS / Record, tapi tetap tampil jelas di layar fisik!
    } else {
        0x00000000 // WDA_NONE: Normal
    };
    unsafe {
        // 1. Set ke root window
        SetWindowDisplayAffinity(hwnd, flag);
        InvalidateRect(hwnd, std::ptr::null(), 1);

        // 2. Set rekursif ke seluruh child HWND (Chromium / WebView2 render surfaces)
        // Ini kunci agar window TIDAK HILANG / TIDAK HITAM di monitor!
        EnumChildWindows(hwnd, enum_child_affinity, flag as isize);
    }
}

#[tauri::command]
fn set_stealth_mode(window: Window, enabled: bool) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        if let Ok(hwnd) = window.hwnd() {
            apply_stealth_recursive(hwnd.0 as isize, enabled);
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
                let win_clone = window.clone();
                // Terapkan setelah window siap agar child WebView2 HWND sudah terbentuk sempurna
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(600));
                    #[cfg(target_os = "windows")]
                    {
                        if let Ok(hwnd) = win_clone.hwnd() {
                            apply_stealth_recursive(hwnd.0 as isize, true);
                        }
                    }
                });
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
