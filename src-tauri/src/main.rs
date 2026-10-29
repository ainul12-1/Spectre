#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{Manager, Window};

#[cfg(target_os = "windows")]
#[link(name = "user32")]
extern "system" {
    fn SetWindowDisplayAffinity(hwnd: isize, dw_affinity: u32) -> i32;
    fn GetWindowLongW(hwnd: isize, n_index: i32) -> i32;
    fn SetWindowLongW(hwnd: isize, n_index: i32, dw_new_long: i32) -> i32;
    fn SetWindowPos(
        hwnd: isize,
        hwnd_insert_after: isize,
        x: i32,
        y: i32,
        cx: i32,
        cy: i32,
        u_flags: u32,
    ) -> i32;
    fn GetAncestor(hwnd: isize, ga_flags: u32) -> isize;
    fn ReleaseCapture() -> i32;
    fn SendMessageW(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> isize;
}

#[cfg(target_os = "windows")]
const GWL_EXSTYLE: i32 = -20;
#[cfg(target_os = "windows")]
const WS_EX_LAYERED: i32 = 0x00080000;
#[cfg(target_os = "windows")]
const SWP_NOSIZE: u32 = 0x0001;
#[cfg(target_os = "windows")]
const SWP_NOMOVE: u32 = 0x0002;
#[cfg(target_os = "windows")]
const SWP_NOZORDER: u32 = 0x0004;
#[cfg(target_os = "windows")]
const SWP_FRAMECHANGED: u32 = 0x0020;
#[cfg(target_os = "windows")]
const GA_ROOT: u32 = 2;

#[cfg(target_os = "windows")]
fn apply_stealth_mode(hwnd: isize, enabled: bool) {
    let top_hwnd = unsafe {
        let root = GetAncestor(hwnd, GA_ROOT);
        if root != 0 {
            root
        } else {
            hwnd
        }
    };

    let flag = if enabled {
        0x00000011 // WDA_EXCLUDEFROMCAPTURE: Gaib di SS / OBS / Record, tetap tampil jelas di layar fisik!
    } else {
        0x00000000 // WDA_NONE: Normal
    };

    unsafe {
        let ex_style = GetWindowLongW(top_hwnd, GWL_EXSTYLE);
        if (ex_style & WS_EX_LAYERED) != 0 {
            SetWindowLongW(top_hwnd, GWL_EXSTYLE, ex_style & !WS_EX_LAYERED);
            SetWindowPos(
                top_hwnd,
                0,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
            );
        }

        SetWindowDisplayAffinity(top_hwnd, flag);
    }
}

static STEALTH_ENABLED: AtomicBool = AtomicBool::new(true);

#[tauri::command]
fn set_stealth_mode(window: Window, enabled: bool) -> Result<bool, String> {
    STEALTH_ENABLED.store(enabled, Ordering::SeqCst);
    #[cfg(target_os = "windows")]
    {
        if let Ok(hwnd) = window.hwnd() {
            apply_stealth_mode(hwnd.0 as isize, enabled);
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

const INJECTED_SAFARI_HEADER_JS: &str = r#"
(function() {
  if (window.__spectre_injected) return;
  window.__spectre_injected = true;

  if (window.location.hostname === 'tauri.localhost' && (window.location.pathname === '/' || window.location.pathname === '/index.html')) {
    return;
  }

  function sendCmd(cmd) {
    var img = new Image();
    img.src = 'http://127.0.0.1:49221/' + cmd + '?t=' + Date.now();
    try {
      fetch('http://127.0.0.1:49221/' + cmd + '?t=' + Date.now(), { mode: 'no-cors' }).catch(function(){});
    } catch(e) {}
  }

  function injectSafariHeader() {
    if (document.getElementById('__spectre_toolbar')) return;

    var style = document.createElement('style');
    style.id = '__spectre_style';
    style.textContent = '' +
      '#__spectre_toolbar {' +
      '  position: fixed !important;' +
      '  top: 0 !important;' +
      '  left: 0 !important;' +
      '  width: 100vw !important;' +
      '  height: 44px !important;' +
      '  background: rgba(30, 31, 42, 0.95) !important;' +
      '  backdrop-filter: blur(20px) !important;' +
      '  -webkit-backdrop-filter: blur(20px) !important;' +
      '  border-bottom: 1px solid rgba(255, 255, 255, 0.12) !important;' +
      '  display: flex !important;' +
      '  align-items: center !important;' +
      '  justify-content: space-between !important;' +
      '  padding: 0 14px !important;' +
      '  z-index: 2147483647 !important;' +
      '  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif !important;' +
      '  box-sizing: border-box !important;' +
      '  user-select: none !important;' +
      '  -webkit-user-select: none !important;' +
      '  color: #fff !important;' +
      '  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.4) !important;' +
      '}' +
      '#__spectre_toolbar * { box-sizing: border-box !important; }' +
      'html, body { margin-top: 44px !important; }';
    (document.head || document.documentElement).appendChild(style);

    var bar = document.createElement('div');
    bar.id = '__spectre_toolbar';

    bar.innerHTML = '' +
      '<div style="display:flex;align-items:center;gap:12px;">' +
      '  <div style="display:flex;align-items:center;gap:8px;padding-right:6px;">' +
      '    <div id="__sp_close" title="Tutup Window" style="width:12px;height:12px;border-radius:50%;background:#ff5f57;cursor:pointer;box-shadow:inset 0 1px 1px rgba(255,255,255,0.4),0 1px 2px rgba(0,0,0,0.35);"></div>' +
      '    <div id="__sp_min" title="Kecilkan Window" style="width:12px;height:12px;border-radius:50%;background:#febc2e;cursor:pointer;box-shadow:inset 0 1px 1px rgba(255,255,255,0.4),0 1px 2px rgba(0,0,0,0.35);"></div>' +
      '    <div id="__sp_max" title="Perbesar Window" style="width:12px;height:12px;border-radius:50%;background:#28c840;cursor:pointer;box-shadow:inset 0 1px 1px rgba(255,255,255,0.4),0 1px 2px rgba(0,0,0,0.35);"></div>' +
      '  </div>' +
      '  <button id="__sp_back" title="Kembali" style="width:28px;height:26px;border-radius:6px;border:none;background:rgba(255,255,255,0.08);color:#fff;cursor:pointer;font-size:16px;line-height:1;display:flex;align-items:center;justify-content:center;">‹</button>' +
      '  <button id="__sp_fwd" title="Maju" style="width:28px;height:26px;border-radius:6px;border:none;background:rgba(255,255,255,0.08);color:#fff;cursor:pointer;font-size:16px;line-height:1;display:flex;align-items:center;justify-content:center;">›</button>' +
      '  <button id="__sp_home" title="Halaman Awal" style="width:28px;height:26px;border-radius:6px;border:none;background:rgba(255,255,255,0.08);color:#fff;cursor:pointer;font-size:13px;display:flex;align-items:center;justify-content:center;">🏠</button>' +
      '</div>' +
      '<form id="__sp_form" style="flex:1;max-width:580px;margin:0 14px;display:flex;align-items:center;">' +
      '  <input id="__sp_addr" type="text" value="' + window.location.href + '" style="width:100%;height:30px;background:rgba(0,0,0,0.4);border:1px solid rgba(255,255,255,0.14);border-radius:8px;color:#fff;font-size:12.5px;padding:0 12px;outline:none;text-align:center;" />' +
      '</form>' +
      '<div style="display:flex;align-items:center;gap:10px;">' +
      '  <div id="__sp_stealth" title="Status Anti-Screenshot" style="height:26px;padding:0 10px;border-radius:999px;border:1px solid rgba(40,200,64,0.35);background:rgba(40,200,64,0.15);color:#4cd964;font-size:11px;font-weight:600;display:flex;align-items:center;gap:6px;cursor:pointer;">' +
      '    <span style="width:6px;height:6px;border-radius:50%;background:#28c840;box-shadow:0 0 6px #28c840;"></span>' +
      '    <span id="__sp_stealth_lbl">Anti-SS: ON</span>' +
      '  </div>' +
      '</div>';

    document.documentElement.appendChild(bar);

    document.getElementById('__sp_close').onclick = function() { sendCmd('close'); };
    document.getElementById('__sp_min').onclick = function() { sendCmd('minimize'); };
    document.getElementById('__sp_max').onclick = function() { sendCmd('maximize'); };
    document.getElementById('__sp_back').onclick = function() { window.history.back(); };
    document.getElementById('__sp_fwd').onclick = function() { window.history.forward(); };
    document.getElementById('__sp_home').onclick = function() { window.location.href = 'https://tauri.localhost/'; };

    document.getElementById('__sp_form').onsubmit = function(e) {
      e.preventDefault();
      var val = document.getElementById('__sp_addr').value.trim();
      if (!val) return;
      if (/^https?:\/\//i.test(val)) {
        window.location.href = val;
      } else if (/^[a-zA-Z0-9-]+(\.[a-zA-Z0-9-]+)+/.test(val) && val.indexOf(' ') === -1) {
        window.location.href = 'https://' + val;
      } else {
        window.location.href = 'https://www.google.com/search?q=' + encodeURIComponent(val);
      }
    };

    var stealthOn = true;
    var btnSt = document.getElementById('__sp_stealth');
    var lblSt = document.getElementById('__sp_stealth_lbl');
    btnSt.onclick = function() {
      stealthOn = !stealthOn;
      sendCmd('toggle-stealth');
      lblSt.textContent = stealthOn ? 'Anti-SS: ON' : 'Anti-SS: OFF';
      btnSt.style.borderColor = stealthOn ? 'rgba(40,200,64,0.35)' : 'rgba(255,95,87,0.35)';
      btnSt.style.background = stealthOn ? 'rgba(40,200,64,0.15)' : 'rgba(255,95,87,0.15)';
      btnSt.style.color = stealthOn ? '#4cd964' : '#ff6b64';
    };

    bar.onmousedown = function(e) {
      if (e.button === 0 && !e.target.closest('input, button, form, #__sp_close, #__sp_min, #__sp_max, #__sp_stealth')) {
        sendCmd('drag');
      }
    };
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', injectSafariHeader);
  } else {
    injectSafariHeader();
  }
})();
"#;

fn start_local_command_server(app_handle: tauri::AppHandle) {
    std::thread::spawn(move || {
        let listener = match TcpListener::bind("127.0.0.1:49221") {
            Ok(l) => l,
            Err(_) => return,
        };
        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                let mut buffer = [0u8; 512];
                if let Ok(size) = stream.read(&mut buffer) {
                    let req = String::from_utf8_lossy(&buffer[..size]);
                    if let Some(win) = app_handle.get_window("main") {
                        if req.contains("GET /close") {
                            let _ = win.close();
                        } else if req.contains("GET /minimize") {
                            let _ = win.minimize();
                        } else if req.contains("GET /maximize") {
                            if win.is_maximized().unwrap_or(false) {
                                let _ = win.unmaximize();
                            } else {
                                let _ = win.maximize();
                            }
                        } else if req.contains("GET /toggle-stealth") {
                            let cur = STEALTH_ENABLED.load(Ordering::SeqCst);
                            let new_val = !cur;
                            STEALTH_ENABLED.store(new_val, Ordering::SeqCst);
                            #[cfg(target_os = "windows")]
                            if let Ok(hwnd) = win.hwnd() {
                                apply_stealth_mode(hwnd.0 as isize, new_val);
                            }
                        } else if req.contains("GET /drag") {
                            #[cfg(target_os = "windows")]
                            if let Ok(hwnd) = win.hwnd() {
                                unsafe {
                                    ReleaseCapture();
                                    SendMessageW(hwnd.0 as isize, 0x00A1, 2, 0);
                                }
                            }
                        }
                    }
                    let resp = "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\nOK";
                    let _ = stream.write_all(resp.as_bytes());
                }
            }
        }
    });
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle();
            start_local_command_server(handle);

            if let Some(window) = app.get_window("main") {
                let win_clone = window.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(600));
                    #[cfg(target_os = "windows")]
                    {
                        if let Ok(hwnd) = win_clone.hwnd() {
                            apply_stealth_mode(hwnd.0 as isize, true);
                        }
                    }
                });
            }
            Ok(())
        })
        .on_page_load(|window, payload| {
            if payload.event() == tauri::PageLoadEvent::Finished {
                let _ = window.eval(INJECTED_SAFARI_HEADER_JS);
            }
        })
        .invoke_handler(tauri::generate_handler![
            set_stealth_mode,
            set_always_on_top
        ])
        .run(tauri::generate_context!())
        .expect("error while running Spectre application");
}
