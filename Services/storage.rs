// Copyright (C) 2026 slatkisasa
// SPDX-License-Identifier: AGPL-3.0-only
use crate::account::Account;
use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

/// A readable account file whose contents cannot be used safely.
#[derive(Debug)]
pub struct DamagedMetadata {
    pub original: Vec<u8>,
}
impl std::fmt::Display for DamagedMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Saved account data is damaged or contains invalid or duplicate records")
    }
}
impl std::error::Error for DamagedMetadata {}

fn validate_accounts(accounts: &[Account]) -> Result<()> {
    let mut ids = std::collections::HashSet::new();
    let mut identifiers: Vec<&str> = vec![];
    for account in accounts {
        let id = &account.account_identifier;
        anyhow::ensure!(
            !account.id.is_nil()
                && !account.display_name.trim().is_empty()
                && !id.trim().is_empty()
                && id == id.trim()
                && !id.contains(',')
                && ids.insert(account.id)
                && !identifiers.iter().any(|other| same_identifier(other, id)),
            "Invalid or duplicate account record"
        );
        identifiers.push(id);
    }
    Ok(())
}

#[derive(Clone)]
pub struct Paths {
    pub data: PathBuf,
    pub config: PathBuf,
    pub launcher: PathBuf,
}
impl Paths {
    pub fn defaults() -> Result<Self> {
        anyhow::ensure!(cfg!(windows), "SessionDeck requires Windows 11 x64.");
        let env = |key| {
            std::env::var_os(key)
                .map(PathBuf::from)
                .with_context(|| format!("{key} is unavailable"))
        };
        Ok(Self {
            data: env("LOCALAPPDATA")?.join("SessionDeck"),
            config: env("APPDATA")?.join("Battle.net/Battle.net.config"),
            launcher: env("ProgramFiles(x86)")?.join("Battle.net/Battle.net Launcher.exe"),
        })
    }
    pub fn load(&self) -> Result<Vec<Account>> {
        let bytes = match fs::read(self.data.join("accounts.json")) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
            Err(e) => return Err(e.into()),
        };
        let json = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(&bytes);
        let parsed = serde_json::from_slice::<Vec<Account>>(json);
        match parsed {
            Ok(accounts) if validate_accounts(&accounts).is_ok() => Ok(accounts),
            _ => Err(DamagedMetadata { original: bytes }.into()),
        }
    }
    pub fn load_or_import(&self) -> Result<Vec<Account>> {
        if self.data.join("accounts.json").try_exists()? {
            return self.load();
        }
        let mut accounts: Vec<Account> = vec![];
        for id in self.remembered()? {
            if !accounts
                .iter()
                .any(|a| same_identifier(&a.account_identifier, &id))
            {
                accounts.push(Account::new(id));
            }
        }
        // Persist even an empty first-use list so future launches never reimport.
        self.save(&accounts)?;
        Ok(accounts)
    }
    pub fn recover_accounts(&self, original: &[u8]) -> Result<PathBuf> {
        let path = self.data.join("accounts.json");
        anyhow::ensure!(
            fs::read(&path)? == original,
            "Saved account file changed; restart SessionDeck before recovering"
        );
        let backup = self
            .data
            .join(format!("accounts.json.damaged-{}", uuid::Uuid::new_v4()));
        let mut preserved = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&backup)?;
        preserved.write_all(original)?;
        preserved.sync_all()?;
        anyhow::ensure!(
            fs::read(&path)? == original,
            "Saved account file changed; recovery cancelled"
        );
        self.save(&[]).with_context(|| {
            format!(
                "Could not reset accounts; original copy preserved at {}",
                backup.display()
            )
        })?;
        Ok(backup)
    }
    pub fn save(&self, accounts: &[Account]) -> Result<()> {
        validate_accounts(accounts)?;
        fs::create_dir_all(&self.data)?;
        atomic_write(
            &self.data.join("accounts.json"),
            &serde_json::to_vec_pretty(accounts)?,
        )
    }
    pub fn remembered(&self) -> Result<Vec<String>> {
        if !self.config.exists() {
            return Ok(vec![]);
        }
        names(&parse(&fs::read(&self.config)?)?)
    }
    pub fn update_config(
        &self,
        account: Option<&str>,
        check_stopped: impl Fn() -> Result<()>,
    ) -> Result<PathBuf> {
        let original = fs::read(&self.config)?;
        let mut root = parse(&original)?;
        let mut accounts = names(&root)?;
        let value = if let Some(id) = account {
            anyhow::ensure!(
                !id.trim().is_empty() && !id.contains(','),
                "Invalid account identifier"
            );
            accounts.retain(|s| !same_identifier(s, id));
            accounts.insert(0, id.trim().into());
            let mut unique: Vec<String> = vec![];
            for s in accounts {
                if !unique.iter().any(|a| same_identifier(a, &s)) {
                    unique.push(s);
                }
            }
            unique.join(",")
        } else {
            String::new()
        };
        root["Client"]["SavedAccountNames"] = Value::String(value);
        let bytes = serde_json::to_vec_pretty(&root)?;
        let mut temp =
            tempfile::NamedTempFile::new_in(self.config.parent().context("Config has no parent")?)?;
        temp.write_all(&bytes)?;
        temp.as_file().sync_all()?;
        let backups = self.data.join("Backups");
        fs::create_dir_all(&backups)?;
        let backup = backups.join(format!(
            "Battle.net.config.{}-{}.backup",
            chrono::Utc::now().format("%Y%m%d-%H%M%S-%f"),
            uuid::Uuid::new_v4()
        ));
        check_stopped()?;
        anyhow::ensure!(
            fs::read(&self.config)? == original,
            "Battle.net configuration changed; retry the operation"
        );
        let mut backup_file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&backup)?;
        backup_file.write_all(&original)?;
        backup_file.sync_all()?;
        check_stopped()?;
        replace(temp, &self.config)?;
        let mut old: Vec<_> = fs::read_dir(backups)?
            .filter_map(Result::ok)
            .filter(|e| {
                let n = e.file_name();
                let n = n.to_string_lossy();
                n.starts_with("Battle.net.config.") && n.ends_with(".backup")
            })
            .collect();
        old.sort_by_key(|e| e.file_name());
        let count = old.len().saturating_sub(20);
        for e in old.into_iter().take(count) {
            let _ = fs::remove_file(e.path());
        }
        Ok(backup)
    }
}
fn parse(bytes: &[u8]) -> Result<Value> {
    let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes);
    let root: Value = serde_json::from_slice(bytes).context("Invalid Battle.net configuration")?;
    anyhow::ensure!(
        root.is_object() && root.get("Client").is_some_and(Value::is_object),
        "Configuration must contain a Client object"
    );
    Ok(root)
}
fn names(root: &Value) -> Result<Vec<String>> {
    match root["Client"].get("SavedAccountNames") {
        None | Some(Value::Null) => Ok(vec![]),
        Some(Value::String(s)) => Ok(s
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect()),
        _ => bail!("SavedAccountNames must be a string"),
    }
}
pub fn same_identifier(a: &str, b: &str) -> bool {
    #[cfg(windows)]
    {
        let a: Vec<u16> = a.encode_utf16().collect();
        let b: Vec<u16> = b.encode_utf16().collect();
        // SAFETY: both UTF-16 buffers remain valid for the explicit lengths.
        unsafe {
            windows_sys::Win32::Globalization::CompareStringOrdinal(
                a.as_ptr(),
                a.len() as i32,
                b.as_ptr(),
                b.len() as i32,
                1,
            ) == 2
        }
    }
    #[cfg(not(windows))]
    {
        a.to_lowercase() == b.to_lowercase()
    }
}
pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut temp = tempfile::NamedTempFile::new_in(path.parent().context("Missing parent")?)?;
    temp.write_all(bytes)?;
    temp.as_file().sync_all()?;
    replace(temp, path)
}
fn replace(temp: tempfile::NamedTempFile, path: &Path) -> Result<()> {
    #[cfg(windows)]
    if path.exists() {
        use std::os::windows::ffi::OsStrExt;
        let target: Vec<_> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        let source: Vec<_> = temp
            .path()
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect();
        // SAFETY: terminated UTF-16 paths are valid; optional pointers are null.
        let ok = unsafe {
            windows_sys::Win32::Storage::FileSystem::ReplaceFileW(
                target.as_ptr(),
                source.as_ptr(),
                std::ptr::null(),
                0,
                std::ptr::null(),
                std::ptr::null(),
            )
        };
        anyhow::ensure!(
            ok != 0,
            "Configuration replacement failed: {}",
            std::io::Error::last_os_error()
        );
        return Ok(());
    }
    temp.persist(path).map_err(|e| e.error)?;
    Ok(())
}
