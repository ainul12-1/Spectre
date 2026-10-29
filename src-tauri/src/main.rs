#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
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
#[link(name = "dwmapi")]
extern "system" {
    fn DwmSetWindowAttribute(
        hwnd: isize,
        dw_attribute: u32,
        pv_attribute: *const std::ffi::c_void,
        cb_attribute: u32,
    ) -> i32;
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
fn apply_rounded_corners(hwnd: isize) {
    let top_hwnd = unsafe {
        let root = GetAncestor(hwnd, GA_ROOT);
        if root != 0 {
            root
        } else {
            hwnd
        }
    };

    unsafe {
        // Windows 11 DWM native corner rounding (DWMWCP_ROUND = 2)
        let round_preference: u32 = 2;
        let _ = DwmSetWindowAttribute(
            top_hwnd,
            33, // DWMWA_WINDOW_CORNER_PREFERENCE
            &round_preference as *const _ as *const std::ffi::c_void,
            4,
        );
    }
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn enum_child_affinity(child: isize, lparam: isize) -> i32 {
    let flag = lparam as u32;
    SetWindowDisplayAffinity(child, flag);
    1 // Continue enumeration
}

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

        // 1. Terapkan pada root window handle
        SetWindowDisplayAffinity(top_hwnd, flag);

        // 2. Terapkan rekursif ke SELURUH child HWND (WebView2 renderers & D3D surfaces)
        EnumChildWindows(top_hwnd, enum_child_affinity, flag as isize);
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
  if (window.__spectre_injected_ready) return;
  window.__spectre_injected_ready = true;

  if (window.location.protocol === 'tauri:' || (window.location.hostname === 'tauri.localhost' || window.location.hostname === 'localhost') && (window.location.pathname === '/' || window.location.pathname === '/index.html')) {
    return;
  }

  function sendCmd(cmd) {
    var img = new Image();
    img.src = 'http://127.0.0.1:49221/' + cmd + '?t=' + Date.now();
    try {
      fetch('http://127.0.0.1:49221/' + cmd + '?t=' + Date.now(), { mode: 'no-cors' }).catch(function(){});
    } catch(e) {}
  }

  function ensureHeader() {
    if (document.getElementById('__spectre_root')) {
      var addr = document.getElementById('__spectre_addr_input');
      if (addr && document.activeElement !== addr && addr.value !== window.location.href) {
        addr.value = window.location.href;
      }
      return;
    }

    if (!document.body && !document.documentElement) return;

    var root = document.createElement('div');
    root.id = '__spectre_root';
    root.style.cssText = 'position:fixed!important;top:0!important;left:0!important;width:100vw!important;height:50px!important;z-index:2147483647!important;font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,sans-serif!important;box-sizing:border-box!important;user-select:none!important;-webkit-user-select:none!important;pointer-events:auto!important;';

    // Toolbar background container
    var bar = document.createElement('div');
    bar.style.cssText = 'width:100%!important;height:100%!important;background:#242533!important;border-bottom:1px solid rgba(255,255,255,0.14)!important;display:flex!important;align-items:center!important;justify-content:space-between!important;padding:0 14px!important;box-sizing:border-box!important;color:#fff!important;box-shadow:0 3px 12px rgba(0,0,0,0.55)!important;';

    // Left controls: Traffic lights + Navigation buttons
    var leftBox = document.createElement('div');
    leftBox.style.cssText = 'display:flex;align-items:center;gap:12px;';

    // Traffic light dots
    var tlBox = document.createElement('div');
    tlBox.style.cssText = 'display:flex;align-items:center;gap:8px;padding-right:8px;';

    var btnClose = document.createElement('div');
    btnClose.title = 'Tutup (Close)';
    btnClose.style.cssText = 'width:12px;height:12px;border-radius:50%;background:#ff5f57;cursor:pointer;box-shadow:inset 0 1px 1px rgba(255,255,255,0.4),0 1px 2px rgba(0,0,0,0.4);';
    btnClose.onclick = function(e) { e.stopPropagation(); sendCmd('close'); };

    var btnMin = document.createElement('div');
    btnMin.title = 'Kecilkan (Minimize)';
    btnMin.style.cssText = 'width:12px;height:12px;border-radius:50%;background:#febc2e;cursor:pointer;box-shadow:inset 0 1px 1px rgba(255,255,255,0.4),0 1px 2px rgba(0,0,0,0.4);';
    btnMin.onclick = function(e) { e.stopPropagation(); sendCmd('minimize'); };

    var btnMax = document.createElement('div');
    btnMax.title = 'Perbesar (Maximize)';
    btnMax.style.cssText = 'width:12px;height:12px;border-radius:50%;background:#28c840;cursor:pointer;box-shadow:inset 0 1px 1px rgba(255,255,255,0.4),0 1px 2px rgba(0,0,0,0.4);';
    btnMax.onclick = function(e) { e.stopPropagation(); sendCmd('maximize'); };

    tlBox.appendChild(btnClose);
    tlBox.appendChild(btnMin);
    tlBox.appendChild(btnMax);
    leftBox.appendChild(tlBox);

    function makeBtn(txt, title, onClick) {
      var b = document.createElement('button');
      b.textContent = txt;
      b.title = title;
      b.style.cssText = 'width:28px;height:28px;border-radius:6px;border:none;background:rgba(255,255,255,0.08);color:#d0d2dc;cursor:pointer;font-size:14px;display:flex;align-items:center;justify-content:center;padding:0;outline:none;transition:background 0.15s;';
      b.onmouseover = function() { b.style.background = 'rgba(255,255,255,0.18)'; };
      b.onmouseout = function() { b.style.background = 'rgba(255,255,255,0.08)'; };
      b.onclick = function(e) { e.stopPropagation(); onClick(); };
      return b;
    }

    leftBox.appendChild(makeBtn('‹', 'Kembali (Back)', function() { window.history.back(); }));
    leftBox.appendChild(makeBtn('›', 'Maju (Forward)', function() { window.history.forward(); }));
    leftBox.appendChild(makeBtn('🔄', 'Muat Ulang (Reload)', function() { window.location.reload(); }));
    leftBox.appendChild(makeBtn('🏠', 'Halaman Awal (Home)', function() { window.location.href = 'https://tauri.localhost/'; }));
    bar.appendChild(leftBox);

    // Center: Address Bar with Lock icon
    var form = document.createElement('form');
    form.style.cssText = 'flex:1;max-width:620px;margin:0 14px;display:flex;align-items:center;background:rgba(0,0,0,0.45);border:1px solid rgba(255,255,255,0.16);border-radius:9px;padding:0 10px;height:32px;box-sizing:border-box;';

    var lock = document.createElement('span');
    lock.textContent = '🔒';
    lock.style.cssText = 'font-size:12px;margin-right:8px;opacity:0.8;';
    form.appendChild(lock);

    var inp = document.createElement('input');
    inp.id = '__spectre_addr_input';
    inp.type = 'text';
    inp.value = window.location.href;
    inp.style.cssText = 'flex:1;height:100%;background:transparent;border:none;color:#fff;font-size:12.5px;outline:none;text-align:center;box-sizing:border-box;';
    inp.onfocus = function() { inp.style.textAlign = 'left'; inp.select(); };
    inp.onblur = function() { inp.style.textAlign = 'center'; };

    form.onsubmit = function(e) {
      e.preventDefault();
      var val = inp.value.trim();
      if (!val) return;
      if (/^https?:\/\//i.test(val)) {
        window.location.href = val;
      } else if (/^[a-zA-Z0-9-]+(\.[a-zA-Z0-9-]+)+/.test(val) && val.indexOf(' ') === -1) {
        window.location.href = 'https://' + val;
      } else {
        window.location.href = 'https://www.google.com/search?q=' + encodeURIComponent(val);
      }
    };
    form.appendChild(inp);
    bar.appendChild(form);

    // Right: Anti-SS Toggle + New Tab
    var rightBox = document.createElement('div');
    rightBox.style.cssText = 'display:flex;align-items:center;gap:10px;';

    var btnStealth = document.createElement('div');
    btnStealth.title = 'Status Anti-Screenshot & Screen-Recording';
    btnStealth.style.cssText = 'height:28px;padding:0 12px;border-radius:999px;border:1px solid rgba(40,200,64,0.35);background:rgba(40,200,64,0.15);color:#4cd964;font-size:11.5px;font-weight:600;display:flex;align-items:center;gap:6px;cursor:pointer;';
    
    var dot = document.createElement('span');
    dot.style.cssText = 'width:7px;height:7px;border-radius:50%;background:#28c840;box-shadow:0 0 8px #28c840;';
    var lbl = document.createElement('span');
    lbl.textContent = 'Anti-SS: ON';

    btnStealth.appendChild(dot);
    btnStealth.appendChild(lbl);

    var stealthOn = true;
    btnStealth.onclick = function(e) {
      e.stopPropagation();
      stealthOn = !stealthOn;
      sendCmd('toggle-stealth');
      lbl.textContent = stealthOn ? 'Anti-SS: ON' : 'Anti-SS: OFF';
      btnStealth.style.borderColor = stealthOn ? 'rgba(40,200,64,0.35)' : 'rgba(255,95,87,0.35)';
      btnStealth.style.background = stealthOn ? 'rgba(40,200,64,0.15)' : 'rgba(255,95,87,0.15)';
      btnStealth.style.color = stealthOn ? '#4cd964' : '#ff6b64';
      dot.style.background = stealthOn ? '#28c840' : '#ff5f57';
      dot.style.boxShadow = stealthOn ? '0 0 8px #28c840' : '0 0 8px #ff5f57';
    };
    rightBox.appendChild(btnStealth);

    // New Tab button
    var btnNewTab = makeBtn('+', 'Buka Halaman Baru', function() {
      window.location.href = 'https://tauri.localhost/';
    });
    rightBox.appendChild(btnNewTab);

    bar.appendChild(rightBox);

    // Drag support
    bar.onmousedown = function(e) {
      if (e.button === 0 && !e.target.closest('input, button, form, div[title]')) {
        sendCmd('drag');
      }
    };

    root.appendChild(bar);
    (document.body || document.documentElement).appendChild(root);

    // Shift website body & masthead down
    var style = document.createElement('style');
    style.id = '__spectre_page_offset';
    style.textContent = 'html, body { padding-top: 50px !important; } #masthead-container { top: 50px !important; } ytd-app { margin-top: 50px !important; }';
    (document.head || document.documentElement).appendChild(style);
  }

  if (document.body || document.documentElement) {
    ensureHeader();
  }
  setInterval(ensureHeader, 400);
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

            let window = tauri::WindowBuilder::new(
                app,
                "main",
                tauri::WindowUrl::App("index.html".into()),
            )
            .title("Spectre")
            .inner_size(1200.0, 800.0)
            .min_inner_size(640.0, 450.0)
            .resizable(true)
            .decorations(false)
            .transparent(false)
            .center()
            .initialization_script(INJECTED_SAFARI_HEADER_JS)
            .build()?;

            let win_clone = window.clone();
            std::thread::spawn(move || {
                for delay in [300, 800, 1800] {
                    std::thread::sleep(std::time::Duration::from_millis(delay));
                    #[cfg(target_os = "windows")]
                    {
                        if let Ok(hwnd) = win_clone.hwnd() {
                            apply_rounded_corners(hwnd.0 as isize);
                            apply_stealth_mode(hwnd.0 as isize, true);
                        }
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            set_stealth_mode,
            set_always_on_top
        ])
        .run(tauri::generate_context!())
        .expect("error while running Spectre application");
}
