// Copyright (C) 2026 slatkisasa
// SPDX-License-Identifier: AGPL-3.0-only
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
#[path = "../Models/account.rs"]
mod account;
#[path = "../UI/app.rs"]
mod app;
#[path = "../Services/launcher.rs"]
mod launcher;
#[path = "../Services/storage.rs"]
mod storage;
#[cfg(test)]
#[path = "../Services/tests.rs"]
mod tests;
#[path = "../Services/updates.rs"]
mod updates;

fn main() {
    if let Err(error) = run() {
        let message = format!("SessionDeck could not start: {error:#}");
        #[cfg(windows)]
        {
            let text: Vec<u16> = message.encode_utf16().chain(Some(0)).collect();
            let title: Vec<u16> = "SessionDeck".encode_utf16().chain(Some(0)).collect();
            // SAFETY: terminated UTF-16 buffers remain valid throughout the call.
            unsafe {
                windows_sys::Win32::UI::WindowsAndMessaging::MessageBoxW(
                    std::ptr::null_mut(),
                    text.as_ptr(),
                    title.as_ptr(),
                    0x10,
                );
            }
        }
        #[cfg(not(windows))]
        eprintln!("{message}");
        std::process::exit(1);
    }
}
fn run() -> anyhow::Result<()> {
    let paths = storage::Paths::defaults()?;
    std::fs::create_dir_all(&paths.data)?;
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(paths.data.join("instance.lock"))?;
    fs2::FileExt::try_lock_exclusive(&lock)
        .map_err(|_| anyhow::anyhow!("SessionDeck is already running"))?;
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_icon(app::icon())
            .with_inner_size([860., 560.])
            .with_min_inner_size([640., 460.]),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "SessionDeck",
        options,
        Box::new(move |cc| Ok(Box::new(app::Deck::new(cc, paths)))),
    )
    .map_err(|e| anyhow::anyhow!("{e}"))?;
    drop(lock);
    Ok(())
}
