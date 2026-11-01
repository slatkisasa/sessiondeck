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
    time::{Duration, Instant},
};
use sysinfo::{ProcessesToUpdate, System};

fn launchers(paths: &Paths) -> Result<Vec<sysinfo::Pid>> {
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::All, true);
    let mut ids = vec![];
    for (pid, p) in system.processes() {
        if p.name()
            .to_string_lossy()
            .eq_ignore_ascii_case("Battle.net.exe")
        {
            let exe = p
                .exe()
                .context("Cannot verify a running Battle.net process; close it manually")?;
            anyhow::ensure!(
                exe.file_name()
                    .is_some_and(|s| s.to_string_lossy().eq_ignore_ascii_case("Battle.net.exe")),
                "Unexpected launcher identity"
            );
            let expected = paths
                .launcher
                .parent()
                .context("Launcher path has no parent")?
                .canonicalize()?;
            let actual = exe
                .parent()
                .context("Process path has no parent")?
                .canonicalize()?;
            anyhow::ensure!(
                actual == expected,
                "Battle.net is running from a different installation; close it manually or select the correct launcher path"
            );
            ids.push(*pid);
        }
    }
    Ok(ids)
}
fn stopped(paths: &Paths, cancel: &AtomicBool) -> Result<()> {
    anyhow::ensure!(!cancel.load(Ordering::Relaxed), "Operation cancelled");
    anyhow::ensure!(
        launchers(paths)?.is_empty(),
        "Battle.net is still running; configuration was not written"
    );
    Ok(())
}
pub fn operate(paths: &Paths, account: Option<&str>, cancel: Arc<AtomicBool>) -> Result<()> {
    anyhow::ensure!(
        paths.launcher.is_file(),
        "Battle.net launcher not found. Select its installation path."
    );
    anyhow::ensure!(paths.config.is_file(), "Battle.net configuration not found");
    anyhow::ensure!(!cancel.load(Ordering::Relaxed), "Operation cancelled");
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::All, true);
    for pid in launchers(paths)? {
        if let Some(p) = system.process(pid) {
            anyhow::ensure!(
                p.kill(),
                "Could not stop Battle.net. Close it manually and retry."
            );
        }
    }
    let deadline = Instant::now() + Duration::from_secs(8);
    while !launchers(paths)?.is_empty() {
        anyhow::ensure!(!cancel.load(Ordering::Relaxed), "Operation cancelled");
        anyhow::ensure!(
            Instant::now() < deadline,
            "Battle.net shutdown timed out; configuration was not written"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
    let backup = paths.update_config(account, || stopped(paths, &cancel))?;
    Command::new(&paths.launcher).spawn().with_context(|| {
        format!(
            "Configuration saved, but launcher could not start. Backup: {}",
            backup.display()
        )
    })?;
    Ok(())
}
