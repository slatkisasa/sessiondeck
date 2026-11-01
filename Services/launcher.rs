// Copyright (C) 2026 slatkisasa
// SPDX-License-Identifier: AGPL-3.0-only
use crate::storage::Paths;
use anyhow::{Context, Result};
use std::{
    process::Command,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use sysinfo::{ProcessesToUpdate, System};

// This snapshot is only used to find candidates and to fail closed before a write.
// Never use sysinfo's module-based exe() query to identify or kill a process:
// it can return an empty path during shutdown and under cross-architecture Windows.
fn launcher_pids() -> Vec<sysinfo::Pid> {
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::All, true);
    system
        .processes()
        .iter()
        .filter_map(|(pid, p)| {
            let name = p.name().to_string_lossy();
            (name.eq_ignore_ascii_case("Battle.net.exe")
                || name.eq_ignore_ascii_case("Battle.net Launcher.exe"))
            .then_some(*pid)
        })
        .collect()
}
fn stopped(cancel: &AtomicBool) -> Result<()> {
    anyhow::ensure!(!cancel.load(Ordering::Relaxed), "Operation cancelled");
    anyhow::ensure!(
        launcher_pids().is_empty(),
        "Battle.net is still running; configuration was not written"
    );
    Ok(())
}

#[cfg(windows)]
mod windows_process {
    use super::*;
    use std::{
        os::windows::ffi::OsStringExt,
        path::{Path, PathBuf},
        time::{Duration, Instant},
    };
    use windows_sys::Win32::{
        Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT},
        System::Threading::{
            OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE, PROCESS_TERMINATE,
            QueryFullProcessImageNameW, TerminateProcess, WaitForSingleObject,
        },
    };

    pub(super) struct Process(HANDLE);
    impl Drop for Process {
        fn drop(&mut self) {
            // SAFETY: this object exclusively owns the successful OpenProcess handle.
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
    impl Process {
        fn open(pid: u32) -> Result<Option<Self>> {
            // SAFETY: scalar rights/PID; ownership of a successful handle is transferred below.
            let handle = unsafe {
                OpenProcess(
                    PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_TERMINATE | PROCESS_SYNCHRONIZE,
                    0,
                    pid,
                )
            };
            if handle.is_null() {
                let error = std::io::Error::last_os_error();
                if error.raw_os_error() == Some(87) {
                    return Ok(None);
                } // PID already gone.
                return Err(error).context("Cannot open Battle.net; close it manually and retry");
            }
            Ok(Some(Self(handle)))
        }
        fn exited(&self) -> Result<bool> {
            // SAFETY: live owned process handle; zero timeout only queries exit state.
            match unsafe { WaitForSingleObject(self.0, 0) } {
                WAIT_OBJECT_0 => Ok(true),
                WAIT_TIMEOUT => Ok(false),
                _ => Err(std::io::Error::last_os_error())
                    .context("Cannot confirm Battle.net shutdown"),
            }
        }
        fn image(&self) -> Result<Option<PathBuf>> {
            if self.exited()? {
                return Ok(None);
            }
            let mut buffer = vec![0u16; 32768];
            let mut length = buffer.len() as u32;
            // SAFETY: checked handle, writable UTF-16 buffer and length for its capacity.
            let ok =
                unsafe { QueryFullProcessImageNameW(self.0, 0, buffer.as_mut_ptr(), &mut length) };
            if ok == 0 {
                let error = std::io::Error::last_os_error();
                if self.exited()? {
                    return Ok(None);
                }
                return Err(error)
                    .context("Cannot verify Battle.net executable; close it manually and retry");
            }
            Ok(Some(PathBuf::from(std::ffi::OsString::from_wide(
                &buffer[..length as usize],
            ))))
        }
        fn terminate(&self) -> Result<()> {
            if self.exited()? {
                return Ok(());
            }
            // SAFETY: owned handle opened with PROCESS_TERMINATE; it cannot refer to a reused PID.
            if unsafe { TerminateProcess(self.0, 0) } == 0 {
                let error = std::io::Error::last_os_error();
                if !self.exited()? {
                    return Err(error)
                        .context("Could not stop Battle.net; close it manually and retry");
                }
            }
            Ok(())
        }
    }
    fn verify_installation(launcher: &Path, executable: &Path) -> Result<()> {
        anyhow::ensure!(
            executable.file_name().is_some_and(|s| {
                let name = s.to_string_lossy();
                name.eq_ignore_ascii_case("Battle.net.exe")
                    || name.eq_ignore_ascii_case("Battle.net Launcher.exe")
            }),
            "Unexpected launcher executable; close Battle.net manually and retry"
        );
        let root = launcher
            .parent()
            .context("Launcher path has no parent")?
            .canonicalize()?;
        let actual = executable.canonicalize()?;
        anyhow::ensure!(
            actual.starts_with(root),
            "Battle.net is running from a different installation; select the correct launcher path"
        );
        Ok(())
    }
    pub(super) fn stop(paths: &Paths, cancel: &AtomicBool) -> Result<()> {
        let mut targets = vec![];
        // Validate all candidates before stopping any. Unknown identity is never permission to kill.
        for pid in launcher_pids() {
            if let Some(process) = Process::open(pid.as_u32())?
                && let Some(image) = process.image()?
            {
                if let Err(error) = verify_installation(&paths.launcher, &image) {
                    if process.exited()? {
                        continue;
                    }
                    return Err(error);
                }
                targets.push(process);
            }
        }
        for process in &targets {
            anyhow::ensure!(!cancel.load(Ordering::Relaxed), "Operation cancelled");
            process.terminate()?;
        }
        let deadline = Instant::now() + Duration::from_secs(8);
        loop {
            anyhow::ensure!(!cancel.load(Ordering::Relaxed), "Operation cancelled");
            let mut all_exited = true;
            for process in &targets {
                all_exited &= process.exited()?;
            }
            if all_exited {
                break;
            }
            anyhow::ensure!(
                Instant::now() < deadline,
                "Battle.net shutdown timed out; configuration was not written"
            );
            std::thread::sleep(Duration::from_millis(100));
        }
        drop(targets);
        stopped(cancel)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn query_image_handles_current_and_exited_processes() {
            let current = Process::open(std::process::id()).unwrap().unwrap();
            assert_eq!(
                current.image().unwrap().unwrap().canonicalize().unwrap(),
                std::env::current_exe().unwrap().canonicalize().unwrap()
            );
            let mut child = Command::new("cmd.exe")
                .args(["/c", "exit", "0"])
                .spawn()
                .unwrap();
            if let Some(handle) = Process::open(child.id()).unwrap() {
                child.wait().unwrap();
                assert!(handle.exited().unwrap());
                assert!(handle.image().unwrap().is_none());
                handle.terminate().unwrap(); // Already exited is harmless.
            } else {
                child.wait().unwrap();
            }
        }
        #[test]
        fn versioned_installation_is_allowed_but_other_directory_is_not() {
            let dir = tempfile::tempdir().unwrap();
            let root = dir.path().join("installation");
            let version = root.join("Battle.net.12345");
            std::fs::create_dir_all(&version).unwrap();
            let launcher = root.join("Battle.net Launcher.exe");
            let executable = version.join("Battle.net.exe");
            std::fs::write(&launcher, []).unwrap();
            std::fs::write(&executable, []).unwrap();
            verify_installation(&launcher, &executable).unwrap();
            let other = dir.path().join("Battle.net.exe");
            std::fs::write(&other, []).unwrap();
            assert!(verify_installation(&launcher, &other).is_err());
        }
    }
}

pub fn operate(paths: &Paths, account: Option<&str>, cancel: Arc<AtomicBool>) -> Result<()> {
    anyhow::ensure!(
        paths.launcher.is_file(),
        "Battle.net launcher not found. Select its installation path."
    );
    anyhow::ensure!(paths.config.is_file(), "Battle.net configuration not found");
    anyhow::ensure!(!cancel.load(Ordering::Relaxed), "Operation cancelled");
    #[cfg(windows)]
    windows_process::stop(paths, &cancel)?;
    #[cfg(not(windows))]
    require_windows()?;
    let backup = paths.update_config(account, || stopped(&cancel))?;
    Command::new(&paths.launcher).spawn().with_context(|| {
        format!(
            "Configuration saved, but launcher could not start. Backup: {}",
            backup.display()
        )
    })?;
    Ok(())
}

#[cfg(not(windows))]
fn require_windows() -> Result<()> {
    anyhow::bail!("Launcher operations require Windows")
}
