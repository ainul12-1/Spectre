#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use std::ffi::c_void;
use tauri::{Manager, Window, WindowEvent};

#[cfg(target_os = "windows")]
const WDA_NONE: u32 = 0x00000000;
#[cfg(target_os = "windows")]
const WDA_EXCLUDEFROMCAPTURE: u32 = 0x00000011;

#[cfg(target_os = "windows")]
const DWMWA_NCRENDERING_POLICY: u32 = 2;
#[cfg(target_os = "windows")]
const DWMNCRP_DISABLED: u32 = 1;
#[cfg(target_os = "windows")]
const DWMWA_WINDOW_CORNER_PREFERENCE: u32 = 33;
#[cfg(target_os = "windows")]
const DWMWCP_DONOTROUND: u32 = 1;
#[cfg(target_os = "windows")]
const DWMWA_BORDER_COLOR: u32 = 34;
#[cfg(target_os = "windows")]
const DWMWA_COLOR_NONE: u32 = 0xFFFFFFFE;

#[cfg(target_os = "windows")]
const SWP_NOSIZE: u32 = 0x0001;
#[cfg(target_os = "windows")]
const SWP_NOZORDER: u32 = 0x0004;
#[cfg(target_os = "windows")]
const SWP_NOACTIVATE: u32 = 0x0010;

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct WinPoint {
    x: i32,
    y: i32,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct WinRect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[derive(Serialize)]
struct CursorWindowState {
    cursor_x: i32,
    cursor_y: i32,
    win_x: i32,
    win_y: i32,
}

#[derive(Serialize, Clone)]
struct MovedPayload {
    x: i32,
    y: i32,
}

#[cfg(target_os = "windows")]
#[link(name = "user32")]
extern "system" {
    fn SetWindowDisplayAffinity(hwnd: isize, dw_affinity: u32) -> i32;
    fn GetCursorPos(lp_point: *mut WinPoint) -> i32;
    fn GetWindowRect(hwnd: isize, lp_rect: *mut WinRect) -> i32;
    fn SetWindowPos(
        hwnd: isize,
        hwnd_insert_after: isize,
        x: i32,
        y: i32,
        cx: i32,
        cy: i32,
        u_flags: u32,
    ) -> i32;
}

#[cfg(target_os = "windows")]
#[link(name = "dwmapi")]
extern "system" {
    fn DwmSetWindowAttribute(
        hwnd: isize,
        dw_attribute: u32,
        pv_attribute: *const c_void,
        cb_attribute: u32,
    ) -> i32;
}

#[cfg(target_os = "windows")]
fn extract_hwnd(window: &Window) -> Result<isize, String> {
    let hwnd = window.hwnd().map_err(|e| e.to_string())?;
    Ok(hwnd.0 as isize)
}

#[cfg(target_os = "windows")]
fn apply_invisible_outer_frame(window: &Window) {
    if let Ok(hwnd) = extract_hwnd(window) {
        unsafe {
            // Remove outer Windows 11 1px DWM border so only the inner Safari cloth window is visible
            let border_none: u32 = DWMWA_COLOR_NONE;
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_BORDER_COLOR,
                &border_none as *const u32 as *const c_void,
                std::mem::size_of::<u32>() as u32,
            );

            let corner_pref: u32 = DWMWCP_DONOTROUND;
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_WINDOW_CORNER_PREFERENCE,
                &corner_pref as *const u32 as *const c_void,
                std::mem::size_of::<u32>() as u32,
            );

            let nc_policy: u32 = DWMNCRP_DISABLED;
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_NCRENDERING_POLICY,
                &nc_policy as *const u32 as *const c_void,
                std::mem::size_of::<u32>() as u32,
            );
        }
    }
}

#[cfg(target_os = "windows")]
fn apply_capture_stealth(window: &Window, enabled: bool) -> Result<bool, String> {
    let _ = window.set_content_protected(enabled);
    let hwnd = extract_hwnd(window)?;
    let flag = if enabled {
        WDA_EXCLUDEFROMCAPTURE
    } else {
        WDA_NONE
    };
    let ok = unsafe { SetWindowDisplayAffinity(hwnd, flag) };
    if ok == 0 {
        Err("Failed to call SetWindowDisplayAffinity".into())
    } else {
        Ok(enabled)
    }
}

#[tauri::command]
fn set_stealth_mode(window: Window, enabled: bool) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        apply_capture_stealth(&window, enabled)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = window.set_content_protected(enabled);
        Ok(enabled)
    }
}

#[tauri::command]
fn set_always_on_top(window: Window, enabled: bool) -> Result<bool, String> {
    window
        .set_always_on_top(enabled)
        .map_err(|e| e.to_string())?;
    Ok(enabled)
}

#[tauri::command]
fn get_cursor_and_window_rect(window: Window) -> Result<CursorWindowState, String> {
    #[cfg(target_os = "windows")]
    {
        let hwnd = extract_hwnd(&window)?;
        let mut pt = WinPoint::default();
        let mut rect = WinRect::default();
        unsafe {
            GetCursorPos(&mut pt as *mut WinPoint);
            GetWindowRect(hwnd, &mut rect as *mut WinRect);
        }
        Ok(CursorWindowState {
            cursor_x: pt.x,
            cursor_y: pt.y,
            win_x: rect.left,
            win_y: rect.top,
        })
    }
    #[cfg(not(target_os = "windows"))]
    {
        let pos = window.outer_position().map_err(|e| e.to_string())?;
        Ok(CursorWindowState {
            cursor_x: pos.x,
            cursor_y: pos.y,
            win_x: pos.x,
            win_y: pos.y,
        })
    }
}

#[tauri::command]
fn move_window_fast(window: Window, x: i32, y: i32) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let hwnd = extract_hwnd(&window)?;
        unsafe {
            SetWindowPos(
                hwnd,
                0,
                x,
                y,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));
        Ok(())
    }
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            if let Some(window) = app.get_window("main") {
                #[cfg(target_os = "windows")]
                {
                    apply_invisible_outer_frame(&window);
                    let _ = apply_capture_stealth(&window, true);
                }
            }
            Ok(())
        })
        .on_window_event(|event| {
            if let WindowEvent::Moved(pos) = event.event() {
                let _ = event.window().emit(
                    "spectre://window-moved",
                    MovedPayload {
                        x: pos.x,
                        y: pos.y,
                    },
                );
            }
        })
        .invoke_handler(tauri::generate_handler![
            set_stealth_mode,
            set_always_on_top,
            get_cursor_and_window_rect,
            move_window_fast
        ])
        .run(tauri::generate_context!())
        .expect("error while running Spectre application");
}
