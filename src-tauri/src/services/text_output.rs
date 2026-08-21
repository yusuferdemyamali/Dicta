#[cfg(target_os = "windows")]
mod windows_output {
    use std::thread;
    use std::time::Duration;
    use windows::Win32::Foundation::{HANDLE, HGLOBAL};
    use windows::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, SetClipboardData,
    };
    use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
    use windows::Win32::System::Ole::CF_UNICODETEXT;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_CONTROL, VK_V,
    };

    fn save_clipboard() -> Option<Vec<u16>> {
        unsafe {
            if OpenClipboard(None).is_err() {
                return None;
            }
            let handle = match GetClipboardData(CF_UNICODETEXT.0 as u32) {
                Ok(h) => h,
                Err(_) => {
                    let _ = CloseClipboard();
                    return None;
                }
            };
            if handle.is_invalid() {
                let _ = CloseClipboard();
                return None;
            }
            // HANDLE -> HGLOBAL conversion (both are *mut c_void wrappers)
            let hglobal = HGLOBAL(handle.0);
            let ptr = GlobalLock(hglobal);
            let result = if ptr.is_null() {
                None
            } else {
                let mut len = 0usize;
                let mut p = ptr as *const u16;
                while *p != 0 {
                    len += 1;
                    p = p.add(1);
                }
                let slice = std::slice::from_raw_parts(ptr as *const u16, len);
                let data = slice.to_vec();
                let _ = GlobalUnlock(hglobal);
                Some(data)
            };
            let _ = CloseClipboard();
            result
        }
    }

    fn restore_clipboard(data: Vec<u16>) -> anyhow::Result<()> {
        unsafe {
            OpenClipboard(None).map_err(|_| anyhow::anyhow!("OpenClipboard failed on restore"))?;
            let _ = EmptyClipboard();
            let len = data.len() + 1;
            let h = GlobalAlloc(GMEM_MOVEABLE, len * 2)
                .map_err(|_| anyhow::anyhow!("GlobalAlloc failed"))?;
            let ptr = GlobalLock(h) as *mut u16;
            if ptr.is_null() {
                let _ = CloseClipboard();
                anyhow::bail!("GlobalLock failed");
            }
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());
            *ptr.add(data.len()) = 0;
            let _ = GlobalUnlock(h);
            let handle = HANDLE(h.0);
            SetClipboardData(CF_UNICODETEXT.0 as u32, Some(handle))
                .map_err(|_| anyhow::anyhow!("SetClipboardData restore failed"))?;
            let _ = CloseClipboard();
            Ok(())
        }
    }

    fn set_clipboard_text(text: &str) -> anyhow::Result<()> {
        let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        unsafe {
            OpenClipboard(None).map_err(|_| anyhow::anyhow!("OpenClipboard failed"))?;
            let _ = EmptyClipboard();
            let bytes = wide.len() * 2;
            let h = GlobalAlloc(GMEM_MOVEABLE, bytes)
                .map_err(|_| anyhow::anyhow!("GlobalAlloc failed"))?;
            let ptr = GlobalLock(h) as *mut u16;
            if ptr.is_null() {
                let _ = CloseClipboard();
                anyhow::bail!("GlobalLock failed");
            }
            std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr, wide.len());
            let _ = GlobalUnlock(h);
            let handle = HANDLE(h.0);
            SetClipboardData(CF_UNICODETEXT.0 as u32, Some(handle))
                .map_err(|_| anyhow::anyhow!("SetClipboardData failed"))?;
            let _ = CloseClipboard();
            Ok(())
        }
    }

    fn send_ctrl_v() -> anyhow::Result<()> {
        unsafe {
            let inputs = [
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VK_CONTROL,
                            wScan: 0,
                            dwFlags: Default::default(),
                            time: 0,
                            ..Default::default()
                        },
                    },
                },
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VK_V,
                            wScan: 0,
                            dwFlags: Default::default(),
                            time: 0,
                            ..Default::default()
                        },
                    },
                },
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VK_V,
                            wScan: 0,
                            dwFlags: KEYEVENTF_KEYUP,
                            time: 0,
                            ..Default::default()
                        },
                    },
                },
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VK_CONTROL,
                            wScan: 0,
                            dwFlags: KEYEVENTF_KEYUP,
                            time: 0,
                            ..Default::default()
                        },
                    },
                },
            ];
            let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            if sent as usize != inputs.len() {
                anyhow::bail!("SendInput sent {}/{}", sent, inputs.len());
            }
            Ok(())
        }
    }

    pub fn insert_text_windows(text: &str) -> anyhow::Result<()> {
        let prev = save_clipboard();
        set_clipboard_text(text)?;
        thread::sleep(Duration::from_millis(80));
        send_ctrl_v()?;
        thread::sleep(Duration::from_millis(150));
        if let Some(prev_data) = prev {
            thread::sleep(Duration::from_millis(200));
            if let Err(e) = restore_clipboard(prev_data) {
                eprintln!("clipboard restore failed (best-effort): {}", e);
            }
        }
        Ok(())
    }
}

#[cfg(target_os = "windows")]
pub fn insert_text(text: &str) -> anyhow::Result<()> {
    windows_output::insert_text_windows(text)
}

#[cfg(not(target_os = "windows"))]
pub fn insert_text(text: &str) -> anyhow::Result<()> {
    match arboard::Clipboard::new() {
        Ok(mut clipboard) => {
            match clipboard.set_text(text.to_string()) {
                Ok(_) => {
                    println!("[dev fallback] text copied to clipboard (Linux): {}", text);
                    Ok(())
                }
                Err(e) => {
                    eprintln!("[dev fallback] clipboard set failed: {}", e);
                    println!("[dev fallback] transcript (stdout): {}", text);
                    Ok(())
                }
            }
        }
        Err(e) => {
            eprintln!("[dev fallback] clipboard init failed: {}", e);
            println!("[dev fallback] transcript: {}", text);
            Ok(())
        }
    }
}
