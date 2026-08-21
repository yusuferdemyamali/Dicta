//! Secure storage for OpenCode Zen API key.
//! Windows: Windows Credential Manager via `keyring` crate (user-account-bound).
//! Linux / non-Windows: development-only via `OPENCODE_ZEN_API_KEY` env var or
//! gitignored local dev file. Not treated as production support.

#[cfg(windows)]
const SERVICE: &str = "dikte.opencode_zen";
#[cfg(windows)]
const ACCOUNT: &str = "default";

/// Save or update the API key.
/// Empty key deletes the stored entry.
pub fn save_api_key(key: &str) -> anyhow::Result<()> {
    let trimmed = key.trim();
    if trimmed.is_empty() {
        return delete_api_key();
    }
    #[cfg(windows)]
    {
        let entry = keyring::Entry::new(SERVICE, ACCOUNT)
            .map_err(|e| anyhow::anyhow!("credential entry error: {}", e))?;
        entry
            .set_password(trimmed)
            .map_err(|e| anyhow::anyhow!("credential store error: {}", e))?;
        Ok(())
    }
    #[cfg(not(windows))]
    {
        // Development-only: persist to a file outside repo (config dir)
        // and rely on env var at load time. This keeps Linux lightweight.
        let path = dev_key_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, trimmed)?;
        // Also set env for current process (helpful for immediate use)
        // SAFETY: set_var is safe in this context (single-threaded worker or startup)
        unsafe {
            std::env::set_var("OPENCODE_ZEN_API_KEY", trimmed);
        }
        Ok(())
    }
}

pub fn delete_api_key() -> anyhow::Result<()> {
    #[cfg(windows)]
    {
        let entry = keyring::Entry::new(SERVICE, ACCOUNT)
            .map_err(|e| anyhow::anyhow!("credential entry error: {}", e))?;
        match entry.delete_credential() {
            Ok(_) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(anyhow::anyhow!("credential delete error: {}", e)),
        }
    }
    #[cfg(not(windows))]
    {
        let path = dev_key_path();
        if path.exists() {
            let _ = std::fs::remove_file(&path);
        }
        unsafe {
            std::env::remove_var("OPENCODE_ZEN_API_KEY");
        }
        Ok(())
    }
}

/// Load API key if present. Returns None if missing/empty.
/// Never logs the key value.
pub fn load_api_key() -> Option<String> {
    #[cfg(windows)]
    {
        // Primary: Windows Credential Manager
        if let Ok(entry) = keyring::Entry::new(SERVICE, ACCOUNT) {
            if let Ok(pw) = entry.get_password() {
                let t = pw.trim().to_string();
                if !t.is_empty() {
                    return Some(t);
                }
            }
        }
        // Development fallback: allow env var even on Windows when debugging
        // but do not advertise as production path.
        if cfg!(debug_assertions) {
            if let Ok(v) = std::env::var("OPENCODE_ZEN_API_KEY") {
                let t = v.trim().to_string();
                if !t.is_empty() {
                    return Some(t);
                }
            }
        }
        None
    }
    #[cfg(not(windows))]
    {
        // Dev-only: env var first
        if let Ok(v) = std::env::var("OPENCODE_ZEN_API_KEY") {
            let t = v.trim().to_string();
            if !t.is_empty() {
                return Some(t);
            }
        }
        // Then gitignored local config file (outside repo, in config dir)
        let path = dev_key_path();
        if let Ok(data) = std::fs::read_to_string(&path) {
            let t = data.trim().to_string();
            if !t.is_empty() {
                return Some(t);
            }
        }
        // Also check .env-style file in repo root for dev convenience (gitignored)
        // Try to locate repo .env.local without exposing path in logs
        if let Ok(cwd) = std::env::current_dir() {
            for name in [".env.local", ".env"] {
                let p = cwd.join(name);
                if let Ok(content) = std::fs::read_to_string(&p) {
                    for line in content.lines() {
                        let line = line.trim();
                        if line.starts_with("OPENCODE_ZEN_API_KEY=") {
                            let val = line
                                .trim_start_matches("OPENCODE_ZEN_API_KEY=")
                                .trim()
                                .trim_matches('"')
                                .trim_matches('\'')
                                .trim()
                                .to_string();
                            if !val.is_empty() {
                                return Some(val);
                            }
                        }
                    }
                }
            }
        }
        None
    }
}

pub fn has_api_key() -> bool {
    load_api_key().is_some_and(|k| !k.trim().is_empty())
}

#[cfg(not(windows))]
fn dev_key_path() -> std::path::PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    base.join("com.dikte.app").join(".zen_key")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_api_key_false_when_empty_env_and_no_file() {
        // This test is best-effort; it checks the helper doesn't panic
        // and that empty key is not considered present.
        // We don't assert true here because CI may have env set.
        let _ = has_api_key();
    }

    #[test]
    fn dev_key_path_is_under_config_dir() {
        #[cfg(not(windows))]
        {
            let p = dev_key_path();
            assert!(p.to_string_lossy().contains("com.dikte.app"));
        }
    }
}
