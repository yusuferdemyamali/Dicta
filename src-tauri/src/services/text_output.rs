#[cfg(target_os = "windows")]
mod windows_output {
    use std::thread;
    use std::time::Duration;
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::System::DataExchange::CF_UNICODETEXT;
    use windows::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, SetClipboardData,
    };
    use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VIRTUAL_KEY,
        VK_CONTROL, VK_V,
    };

    fn save_clipboard() -> Option<Vec<u16>> {
        unsafe {
            if OpenClipboard(HANDLE(0)).is_err() {
                return None;
            }
            let handle = GetClipboardData(CF_UNICODETEXT.0 as u32);
            let result = if let Ok(h) = handle {
                if h.is_invalid() {
                    None
                } else {
                    let ptr = GlobalLock(h);
                    if ptr.is_null() {
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
                        GlobalUnlock(h);
                        Some(data)
                    }
                }
            } else {
                None
            };
            let _ = CloseClipboard();
            result
        }
    }

    fn restore_clipboard(data: Vec<u16>) -> anyhow::Result<()> {
        unsafe {
            if OpenClipboard(HANDLE(0)).is_err() {
                anyhow::bail!("OpenClipboard failed on restore");
            }
            let _ = EmptyClipboard();
            let len = data.len() + 1;
            let h = GlobalAlloc(GMEM_MOVEABLE, len * 2);
            if h.is_invalid() {
                let _ = CloseClipboard();
                anyhow::bail!("GlobalAlloc failed");
            }
            let ptr = GlobalLock(h) as *mut u16;
            if ptr.is_null() {
                let _ = CloseClipboard();
                anyhow::bail!("GlobalLock failed");
            }
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());
            *ptr.add(data.len()) = 0;
            GlobalUnlock(h);
            let res = SetClipboardData(CF_UNICODETEXT.0 as u32, HANDLE(h.0));
            let _ = CloseClipboard();
            if res.is_err() {
                anyhow::bail!("SetClipboardData restore failed");
            }
            Ok(())
        }
    }

    fn set_clipboard_text(text: &str) -> anyhow::Result<()> {
        let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        unsafe {
            if OpenClipboard(HANDLE(0)).is_err() {
                anyhow::bail!("OpenClipboard failed");
            }
            let _ = EmptyClipboard();
            let bytes = wide.len() * 2;
            let h = GlobalAlloc(GMEM_MOVEABLE, bytes);
            if h.is_invalid() {
                let _ = CloseClipboard();
                anyhow::bail!("GlobalAlloc failed");
            }
            let ptr = GlobalLock(h) as *mut u16;
            if ptr.is_null() {
                let _ = CloseClipboard();
                anyhow::bail!("GlobalLock failed");
            }
            std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr, wide.len());
            GlobalUnlock(h);
            let res = SetClipboardData(CF_UNICODETEXT.0 as u32, HANDLE(h.0));
            let _ = CloseClipboard();
            if res.is_err() {
                anyhow::bail!("SetClipboardData failed");
            }
            Ok(())
        }
    }

    fn send_ctrl_v() -> anyhow::Result<()> {
        unsafe {
            let mut inputs = [
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VK_CONTROL,
                            wScan: 0,
                            dwFlags: Default::default(),
                            time: 0,
                            wExtraInfo: 0,
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
                            wExtraInfo: 0,
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
                            wExtraInfo: 0,
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
                            wExtraInfo: 0,
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
        // Preserve clipboard best-effort
        let prev = save_clipboard();

        set_clipboard_text(text)?;
        // Small delay to let clipboard settle
        thread::sleep(Duration::from_millis(80));
        send_ctrl_v()?;
        // Wait a bit before restoring so target app can paste
        thread::sleep(Duration::from_millis(150));

        if let Some(prev_data) = prev {
            // Best-effort restore; delay a bit more
            thread::sleep(Duration::from_millis(200));
            // Spawn restore on separate thread to not block? Do best-effort inline but ignore errors
            if let Err(e) = restore_clipboard(prev_data) {
                eprintln!("clipboard restore failed (best-effort): {}", e);
            }
        }
        Ok(())
    }
}

#[cfg(target_os = "windows")]
pub fn insert_text(text: &str) -> anyhow::Result<()> {
    // Ensure not foregrounding settings window: we never show it here.
    // Preserve Turkish Unicode via clipboard Unicode text
    windows_output::insert_text_windows(text)
}

#[cfg(not(target_os = "windows"))]
pub fn insert_text(text: &str) -> anyhow::Result<()> {
    // Linux development-only fallback: try arboard clipboard, else log
    // This is not product support, just dev-safe fallback without claiming Linux parity.

    // Try arboard
    match arboard::Clipboard::new() {
        Ok(mut clipboard) => {
            match clipboard.set_text(text.to_string()) {
                Ok(_) => {
                    println!("[dev fallback] text copied to clipboard (Linux): {}", text);
                    // On Linux we don't attempt SendInput Ctrl+V as it depends on X11/Wayland
                    // Instead just inform via log and try to keep flow
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
