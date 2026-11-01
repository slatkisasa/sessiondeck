// Copyright (C) 2026 slatkisasa
// SPDX-License-Identifier: AGPL-3.0-only
use crate::{
    account::Account,
    storage::{DamagedMetadata, Paths, same_identifier},
};
use eframe::egui::{self, Color32, RichText};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::time::{Duration, Instant};
use uuid::Uuid;

const BACKGROUND: Color32 = Color32::from_rgb(18, 18, 23);
const SURFACE: Color32 = Color32::from_rgb(25, 25, 32);
const BORDER: Color32 = Color32::from_rgb(47, 47, 58);
const ACCENT: Color32 = Color32::from_rgb(231, 176, 207);
const TEXT: Color32 = Color32::from_rgb(237, 233, 242);
const CURRENT: Color32 = Color32::from_rgb(132, 210, 165);
const MUTED: Color32 = Color32::from_rgb(153, 151, 168);

fn configure_theme(ctx: &egui::Context) {
    // Pin the preference: set_visuals alone still lets Windows switch to light mode.
    ctx.set_theme(egui::ThemePreference::Dark);
    ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(egui::SystemTheme::Dark));
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.override_text_color = Some(TEXT);
    style.visuals.panel_fill = BACKGROUND;
    style.visuals.window_fill = SURFACE;
    style.visuals.window_stroke = egui::Stroke::new(1.0_f32, BORDER);
    style.visuals.selection.bg_fill = Color32::from_rgb(71, 48, 69);
    style.visuals.selection.stroke.color = TEXT;
    style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(37, 37, 46);
    style.visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(37, 37, 46);
    style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, BORDER);
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(58, 47, 62);
    style.visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(58, 47, 62);
    style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0_f32, ACCENT);
    style.spacing.item_spacing = egui::vec2(10., 8.);
    style.spacing.button_padding = egui::vec2(14., 10.);
    style
        .text_styles
        .insert(egui::TextStyle::Body, egui::FontId::proportional(14.));
    style
        .text_styles
        .insert(egui::TextStyle::Button, egui::FontId::proportional(13.));
    ctx.set_style(style);
}
fn primary_button(text: &str, size: [f32; 2]) -> egui::Button<'_> {
    egui::Button::new(RichText::new(text).color(BACKGROUND).strong())
        .fill(ACCENT)
        .stroke(egui::Stroke::NONE)
        .corner_radius(8)
        .min_size(egui::vec2(size[0], size[1]))
}
fn secondary_button(text: &str) -> egui::Button<'_> {
    egui::Button::new(text)
        .corner_radius(7)
        .min_size(egui::vec2(0., 36.))
}

fn account_row(
    ui: &mut egui::Ui,
    account: &Account,
    selected: bool,
    current: bool,
) -> egui::Response {
    let last = account
        .last_used_at
        .map(|t| {
            t.with_timezone(&chrono::Local)
                .format("%b %d, %H:%M")
                .to_string()
        })
        .unwrap_or_else(|| "Not opened yet".into());
    // Keep each text line independent: truncating a multiline Button loses every
    // line after the first newline, even when the card itself has ample space.
    let response = ui.add_sized(
        [ui.available_width(), 88.],
        egui::Button::new("")
            .selected(selected)
            .fill(if selected {
                Color32::from_rgb(49, 37, 53)
            } else {
                Color32::from_rgb(30, 30, 37)
            })
            .stroke(egui::Stroke::new(
                1.0_f32,
                if selected { ACCENT } else { BORDER },
            ))
            .corner_radius(8),
    );
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::SelectableLabel,
            ui.is_enabled(),
            selected,
            format!(
                "{}, {}, last opened: {}{}",
                account.display_name,
                account.account_identifier,
                last,
                if current {
                    ", Current account recorded by Battle.net"
                } else {
                    ""
                }
            ),
        )
    });
    let rect = response.rect;
    let width = (rect.width() - 32.).max(0.);
    let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
    for (text, y, size, color, reserved) in [
        (
            account.display_name.as_str(),
            12.,
            16.,
            TEXT,
            if selected || current { 88. } else { 0. },
        ),
        (account.account_identifier.as_str(), 36., 13., MUTED, 0.),
    ] {
        let mut job = egui::text::LayoutJob::simple(
            text.into(),
            egui::FontId::proportional(size),
            if ui.is_enabled() { color } else { MUTED },
            (width - reserved).max(0.),
        );
        job.wrap.max_rows = 1;
        let galley = painter.layout_job(job);
        painter.galley(rect.min + egui::vec2(16., y), galley, color);
    }
    painter.text(
        rect.min + egui::vec2(16., 61.),
        egui::Align2::LEFT_TOP,
        format!("Last opened: {last}"),
        egui::FontId::proportional(11.),
        MUTED,
    );
    if current {
        painter.circle_filled(
            egui::pos2(rect.right() - 70., rect.top() + 21.),
            3.,
            CURRENT,
        );
        painter.text(
            egui::pos2(rect.right() - 16., rect.top() + 15.),
            egui::Align2::RIGHT_TOP,
            "Current",
            egui::FontId::proportional(11.),
            CURRENT,
        );
    } else if selected {
        painter.text(
            egui::pos2(rect.right() - 16., rect.top() + 15.),
            egui::Align2::RIGHT_TOP,
            "Selected",
            egui::FontId::proportional(11.),
            ACCENT,
        );
    }
    response.on_hover_text(format!(
        "{}\n{}\nLast opened: {}{}",
        account.display_name,
        account.account_identifier,
        last,
        if current {
            "\nCurrent: the last account recorded by Battle.net. Sign-in may still be required."
        } else {
            ""
        }
    ))
}

pub fn icon() -> egui::IconData {
    let mut rgba = Vec::with_capacity(64 * 64 * 4);
    for y in 0..64 {
        for x in 0..64 {
            let back = (12..43).contains(&x) && (12..44).contains(&y);
            let front = (22..53).contains(&x) && (22..54).contains(&y);
            let color = if front {
                ACCENT
            } else if back {
                Color32::from_rgb(123, 99, 143)
            } else {
                BACKGROUND
            };
            rgba.extend_from_slice(&color.to_array());
        }
    }
    egui::IconData {
        rgba,
        width: 64,
        height: 64,
    }
}

pub struct Deck {
    paths: Paths,
    accounts: Vec<Account>,
    selected: Option<Uuid>,
    current: Option<String>,
    current_receiver: Option<mpsc::Receiver<anyhow::Result<Option<String>>>>,
    next_current_refresh: Instant,
    status: String,
    receiver: Option<mpsc::Receiver<anyhow::Result<()>>>,
    switching: Option<Uuid>,
    cancel: Arc<AtomicBool>,
    dialog: Option<Dialog>,
    name: String,
    identifier: String,
    settings: bool,
    launcher_path: String,
    storage_ok: bool,
    damaged: Option<Vec<u8>>,
}
enum Dialog {
    Capture,
    Rename,
    Remove,
    SignIn,
    Recover,
}
impl Deck {
    pub fn new(cc: &eframe::CreationContext<'_>, paths: Paths) -> Self {
        configure_theme(&cc.egui_ctx);
        Self::load(paths)
    }
    fn load(paths: Paths) -> Self {
        let loaded = paths.load_or_import();
        let damaged = loaded
            .as_ref()
            .err()
            .and_then(|e| e.downcast_ref::<DamagedMetadata>())
            .map(|e| e.original.clone());
        let storage_ok = loaded.is_ok();
        let mut status = String::from("Ready • Sessions stay with Battle.net.");
        let accounts = loaded.unwrap_or_else(|e| {
            status = format!("Cannot load accounts: {e}");
            vec![]
        });
        let recovery_needed = damaged.is_some();
        let mut deck = Self {
            launcher_path: paths.launcher.to_string_lossy().into(),
            paths,
            accounts,
            selected: None,
            current: None,
            current_receiver: None,
            next_current_refresh: Instant::now(),
            status,
            receiver: None,
            switching: None,
            cancel: Arc::new(AtomicBool::new(false)),
            dialog: recovery_needed.then_some(Dialog::Recover),
            name: String::new(),
            identifier: String::new(),
            settings: false,
            storage_ok,
            damaged,
        };
        deck.sort();
        deck
    }
    fn invalidate_current(&mut self) {
        self.current = None;
        // Discard an in-flight read started before a switch, so its stale result
        // cannot overwrite the new preference after the operation completes.
        self.current_receiver = None;
        self.next_current_refresh = Instant::now();
    }
    fn refresh_current(&mut self, ctx: &egui::Context) {
        let result = self
            .current_receiver
            .as_ref()
            .and_then(|rx| match rx.try_recv() {
                Ok(result) => Some(result),
                Err(mpsc::TryRecvError::Empty) => None,
                Err(mpsc::TryRecvError::Disconnected) => {
                    Some(Err(anyhow::anyhow!("Current account read interrupted")))
                }
            });
        if let Some(result) = result {
            self.current = result.ok().flatten(); // Unknown is preferable to a stale marker.
            self.current_receiver = None;
        }
        if self.receiver.is_none()
            && self.current_receiver.is_none()
            && Instant::now() >= self.next_current_refresh
        {
            let paths = self.paths.clone();
            let ctx = ctx.clone();
            let (tx, rx) = mpsc::channel();
            self.current_receiver = Some(rx);
            self.next_current_refresh = Instant::now() + Duration::from_secs(3);
            std::thread::spawn(move || {
                let result = paths
                    .remembered()
                    .map(|accounts| accounts.into_iter().next());
                let _ = tx.send(result);
                ctx.request_repaint();
            });
        }
        ctx.request_repaint_after(Duration::from_secs(3));
    }
    fn sort(&mut self) {
        self.accounts.sort_by(|a, b| {
            b.last_used_at.cmp(&a.last_used_at).then_with(|| {
                a.display_name
                    .to_lowercase()
                    .cmp(&b.display_name.to_lowercase())
            })
        });
        if !self.accounts.iter().any(|a| Some(a.id) == self.selected) {
            self.selected = self.accounts.first().map(|a| a.id);
        }
    }
    fn persist(&mut self, accounts: Vec<Account>) -> bool {
        match self.paths.save(&accounts) {
            Ok(()) => {
                self.accounts = accounts;
                self.sort();
                true
            }
            Err(e) => {
                self.status = format!("Could not save accounts: {e}");
                false
            }
        }
    }
    fn capture(&mut self) {
        match self.paths.remembered() {
            Ok(ids) if !ids.is_empty() => {
                self.identifier = ids[0].clone();
                self.name = self
                    .accounts
                    .iter()
                    .find(|a| same_identifier(&a.account_identifier, &self.identifier))
                    .map(|a| a.display_name.clone())
                    .unwrap_or_else(|| self.identifier.split('@').next().unwrap_or("").into());
                self.dialog = Some(Dialog::Capture);
            }
            Ok(_) => {
                self.status = "Finish signing in to Battle.net, close it, then Save current.".into()
            }
            Err(e) => self.status = format!("Could not read current account: {e}"),
        }
    }
    fn start(&mut self, id: Option<Uuid>, ctx: &egui::Context) {
        self.invalidate_current();
        let account = id
            .and_then(|id| self.accounts.iter().find(|a| a.id == id))
            .map(|a| a.account_identifier.clone());
        let paths = self.paths.clone();
        let cancel = self.cancel.clone();
        let (tx, rx) = mpsc::channel();
        let ctx = ctx.clone();
        self.receiver = Some(rx);
        self.switching = id;
        self.status = "Stopping Battle.net → saving preference → starting launcher…".into();
        std::thread::spawn(move || {
            let result = crate::launcher::operate(&paths, account.as_deref(), cancel);
            let _ = tx.send(result);
            ctx.request_repaint();
        });
    }
}
impl Drop for Deck {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}
impl Deck {
    fn render(&mut self, ctx: &egui::Context) {
        let result = self.receiver.as_ref().and_then(|rx| match rx.try_recv() {
            Ok(result) => Some(result),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => Some(Err(anyhow::anyhow!(
                "Operation worker stopped unexpectedly"
            ))),
        });
        if let Some(result) = result {
            self.invalidate_current();
            self.receiver = None;
            match result {
                Ok(()) => {
                    self.status = if self.switching.is_some() {
                        "Battle.net opened with the selected account preference.".into()
                    } else {
                        "Sign in to Battle.net with Stay logged in enabled, then return here and click Save current.".into()
                    };
                    if let Some(id) = self.switching.take() {
                        let mut accounts = self.accounts.clone();
                        if let Some(a) = accounts.iter_mut().find(|a| a.id == id) {
                            a.last_used_at = Some(chrono::Utc::now());
                        }
                        self.persist(accounts);
                    }
                }
                Err(e) => self.status = format!("Operation failed: {e:#}"),
            }
        }
        self.refresh_current(ctx);
        let busy = self.receiver.is_some();
        let enabled = !busy && self.dialog.is_none() && self.storage_ok;
        let selected = self
            .accounts
            .iter()
            .find(|a| Some(a.id) == self.selected)
            .cloned();
        egui::TopBottomPanel::bottom("status")
            .frame(
                egui::Frame::new()
                    .fill(BACKGROUND)
                    .inner_margin(egui::Margin::symmetric(24, 12)),
            )
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    if busy {
                        ui.spinner();
                    } else {
                        let (rect, _) =
                            ui.allocate_exact_size(egui::vec2(10., 16.), egui::Sense::hover());
                        ui.painter().circle_filled(
                            rect.center(),
                            3.,
                            if self.storage_ok {
                                ACCENT
                            } else {
                                Color32::from_rgb(240, 160, 140)
                            },
                        );
                    }
                    ui.label(RichText::new(&self.status).size(12.).color(MUTED));
                });
            });
        egui::TopBottomPanel::bottom("actions")
            .frame(
                egui::Frame::new()
                    .fill(SURFACE)
                    .inner_margin(egui::Margin::symmetric(24, 18))
                    .stroke(egui::Stroke::new(1.0_f32, BORDER)),
            )
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.allocate_ui_with_layout(
                        egui::vec2((ui.available_width() - 355.).max(120.), 42.),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.label(
                                RichText::new("SELECTED ACCOUNT")
                                    .monospace()
                                    .size(10.)
                                    .color(MUTED),
                            );
                            ui.add(
                                egui::Label::new(
                                    RichText::new(
                                        selected
                                            .as_ref()
                                            .map(|a| a.display_name.as_str())
                                            .unwrap_or("Choose an account to switch"),
                                    )
                                    .size(16.)
                                    .strong(),
                                )
                                .truncate(),
                            );
                        },
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add_enabled(
                                enabled && selected.is_some(),
                                primary_button("Switch account", [160., 42.]),
                            )
                            .clicked()
                        {
                            self.start(self.selected, ctx);
                        }
                        if ui
                            .add_enabled(enabled && selected.is_some(), secondary_button("Remove"))
                            .clicked()
                        {
                            self.dialog = Some(Dialog::Remove);
                        }
                        if ui
                            .add_enabled(enabled && selected.is_some(), secondary_button("Rename"))
                            .clicked()
                        {
                            self.name = selected.as_ref().unwrap().display_name.clone();
                            self.dialog = Some(Dialog::Rename);
                        }
                    });
                });
            });
        egui::TopBottomPanel::top("header")
            .frame(
                egui::Frame::new()
                    .fill(BACKGROUND)
                    .inner_margin(egui::Margin::symmetric(24, 20)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("//").monospace().size(28.).color(ACCENT));
                    ui.label(RichText::new("sessiondeck").monospace().size(25.).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add_enabled(!busy, secondary_button("Settings"))
                            .clicked()
                        {
                            self.settings = !self.settings;
                        }
                    });
                });
                ui.label(RichText::new("Your accounts. One place. Back to the game.").color(MUTED));
                ui.add_space(22.);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("YOUR DECK")
                            .monospace()
                            .size(11.)
                            .color(MUTED),
                    );
                    egui::Frame::new()
                        .fill(SURFACE)
                        .corner_radius(5)
                        .inner_margin(egui::Margin::symmetric(8, 3))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(self.accounts.len().to_string())
                                    .size(11.)
                                    .color(ACCENT),
                            );
                        });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add_enabled(enabled, secondary_button("+ Add account"))
                            .on_hover_text("Open Battle.net sign-in, then use Save current to add the account to your deck")
                            .clicked()
                        {
                            self.dialog = Some(Dialog::SignIn);
                        }
                        if ui
                            .add_enabled(enabled, secondary_button("Save current"))
                            .on_hover_text("Save the account currently selected in Battle.net")
                            .clicked()
                        {
                            self.capture();
                        }
                    });
                });
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(BACKGROUND).inner_margin(egui::Margin::symmetric(24, 8)))
            .show(ctx, |ui| {
                let available = ui.available_size();
                egui::Frame::new().fill(SURFACE).stroke(egui::Stroke::new(1.0_f32, BORDER)).corner_radius(12).inner_margin(16).show(ui, |ui| {
                    ui.set_min_size((available - egui::vec2(34., 34.)).max(egui::vec2(0., 0.)));
                    if self.accounts.is_empty() {
                        let height = ui.available_height();
                        ui.vertical_centered(|ui| {
                            ui.add_space((height * 0.5 - 100.).max(8.));
                            if height > 230. {
                                ui.label(RichText::new("//").monospace().size(36.).color(ACCENT));
                                ui.add_space(10.);
                            }
                            ui.label(RichText::new(if self.storage_ok { "Your next session starts here" } else { "Your saved accounts need attention" }).size(21.).strong());
                            ui.add_space(8.);
                            ui.label(RichText::new(if self.storage_ok { "Sign in to Battle.net, then save your current account." } else { "Your account file has been left unchanged. See the status below." }).size(13.).color(MUTED));
                            ui.add_space(18.);
                            if self.damaged.is_some() {
                                if ui.add_enabled(self.dialog.is_none(), primary_button("Recover saved accounts", [190., 40.])).clicked() { self.dialog = Some(Dialog::Recover); }
                            } else if ui.add_enabled(enabled, primary_button("+ Add your first account", [190., 40.])).clicked() { self.dialog = Some(Dialog::SignIn); }
                        });
                    } else {
                        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                            ui.add_enabled_ui(enabled, |ui| {
                                for account in &self.accounts {
                                    let is_selected = Some(account.id) == self.selected;
                                    let is_current = self.current.as_deref().is_some_and(|id| same_identifier(id, &account.account_identifier));
                                    let response = account_row(ui, account, is_selected, is_current);
                                    if response.clicked() { self.selected = Some(account.id); }
                                    ui.add_space(8.);
                                }
                            });
                        });
                    }
                });
            });
        if self.settings {
            egui::Window::new("Settings")
                .open(&mut self.settings)
                .collapsible(false)
                .resizable(true)
                .default_width(420.)
                .show(ctx, |ui| {
                    ui.label(RichText::new("Battle.net launcher").strong());
                    ui.label(
                        RichText::new("Choose a different installation for this run.").color(MUTED),
                    );
                    ui.add_space(8.);
                    ui.add(
                        egui::TextEdit::singleline(&mut self.launcher_path)
                            .desired_width(f32::INFINITY),
                    );
                    ui.add_space(10.);
                    if ui
                        .add_enabled(!busy, secondary_button("Use this path"))
                        .clicked()
                    {
                        self.paths.launcher = self.launcher_path.clone().into();
                        self.status = "Launcher location updated for this run.".into();
                    }
                });
        }
        if self.dialog.is_some() {
            let mut accept = false;
            let mut dismiss = false;
            let title = match self.dialog.as_ref().unwrap() {
                Dialog::SignIn => "Add another account",
                Dialog::Capture => "Save current account",
                Dialog::Rename => "Rename account",
                Dialog::Remove => "Remove account",
                Dialog::Recover => "Recover saved accounts",
            };
            egui::Window::new(title).collapsible(false).resizable(false).default_width(420.).show(ctx, |ui| {
                match self.dialog.as_ref().unwrap() {
                    Dialog::Recover => { ui.label("Saved accounts are damaged or contain invalid records.
Preserve a copy and start with an empty account list?
Use Save current to add accounts again.
Cancel leaves the original file unchanged."); },
                    Dialog::Capture | Dialog::Rename => { ui.label("Account label"); ui.text_edit_singleline(&mut self.name); },
                    Dialog::Remove => { ui.label("Remove this label from SessionDeck? The Battle.net session stays intact."); },
                    Dialog::SignIn => {
                        ui.label("SessionDeck will close Battle.net and reopen its sign-in screen.");
                        ui.add_space(10.);
                        ui.label(RichText::new("Then follow these steps:").strong());
                        ui.label("1. In Battle.net, enable Stay logged in and sign in to the account you want to add.");
                        ui.label("2. Once Battle.net finishes signing in, return to SessionDeck.");
                        ui.label("3. Click Save current, choose a name, then save the account.");
                        ui.add_space(8.);
                        ui.label(RichText::new("Opening the sign-in screen alone does not save an account.").color(MUTED));
                    },
                }
                ui.horizontal(|ui| { accept = ui.add_enabled(!matches!(self.dialog, Some(Dialog::Capture | Dialog::Rename)) || !self.name.trim().is_empty(), egui::Button::new(match self.dialog.as_ref().unwrap() {
                    Dialog::SignIn => "Open sign-in screen",
                    Dialog::Capture => "Save account",
                    Dialog::Rename => "Save name",
                    Dialog::Remove => "Remove account",
                    Dialog::Recover => "Preserve and reset",
                })).clicked(); dismiss = ui.button("Cancel").clicked(); });
            });
            if accept {
                match self.dialog.take().unwrap() {
                    Dialog::SignIn => self.start(None, ctx),
                    Dialog::Recover => {
                        if let Some(original) = &self.damaged {
                            match self.paths.recover_accounts(original) {
                                Ok(backup) => {
                                    self.accounts.clear();
                                    self.selected = None;
                                    self.storage_ok = true;
                                    self.damaged = None;
                                    self.status = format!(
                                        "Accounts reset. Original preserved at {}",
                                        backup.display()
                                    );
                                }
                                Err(e) => {
                                    self.status = format!("Could not recover accounts: {e:#}")
                                }
                            }
                        }
                    }
                    mode => {
                        let message = match &mode {
                            Dialog::Capture => {
                                "Account saved. Select it and click Switch account when you want to use it."
                            }
                            Dialog::Rename => "Account renamed.",
                            Dialog::Remove => {
                                "Account removed from your deck. Its Battle.net session is unchanged."
                            }
                            _ => unreachable!(),
                        };
                        let mut accounts = self.accounts.clone();
                        match mode {
                            Dialog::Capture => {
                                if let Some(a) = accounts.iter_mut().find(|a| {
                                    same_identifier(&a.account_identifier, &self.identifier)
                                }) {
                                    a.display_name = self.name.trim().into();
                                } else {
                                    let mut a = Account::new(self.identifier.clone());
                                    a.display_name = self.name.trim().into();
                                    self.selected = Some(a.id);
                                    accounts.push(a);
                                }
                            }
                            Dialog::Rename => {
                                if let Some(a) =
                                    accounts.iter_mut().find(|a| Some(a.id) == self.selected)
                                {
                                    a.display_name = self.name.trim().into();
                                }
                            }
                            Dialog::Remove => accounts.retain(|a| Some(a.id) != self.selected),
                            _ => unreachable!(),
                        }
                        if self.persist(accounts) {
                            self.status = message.into();
                        }
                    }
                }
            } else if dismiss {
                self.dialog = None;
            }
        }
    }
}

impl eframe::App for Deck {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.render(ctx);
    }
}

#[cfg(test)]
mod layout_tests {
    use super::*;

    #[test]
    fn windows_light_theme_does_not_override_dark_palette() {
        let ctx = egui::Context::default();
        configure_theme(&ctx);
        let input = egui::RawInput {
            system_theme: Some(egui::Theme::Light),
            ..Default::default()
        };
        let _ = ctx.run(input, |ctx| {
            assert!(ctx.style().visuals.dark_mode);
            assert_eq!(ctx.style().visuals.panel_fill, BACKGROUND);
        });
    }

    #[test]
    fn account_card_shows_short_name_identifier_and_last_opened_separately() {
        let dir = tempfile::tempdir().unwrap();
        let paths = Paths {
            data: dir.path().join("metadata"),
            config: dir.path().join("config.json"),
            launcher: dir.path().join("launcher.exe"),
        };
        let mut account = Account::new("synthetic@example.invalid".into());
        account.display_name = "Main account".into();
        paths.save(&[account]).unwrap();
        let ctx = egui::Context::default();
        configure_theme(&ctx);
        let mut deck = Deck::load(paths);
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(860., 560.),
                )),
                ..Default::default()
            },
            |ctx| deck.render(ctx),
        );
        let texts: Vec<_> = output
            .shapes
            .iter()
            .filter_map(|shape| {
                if let egui::Shape::Text(text) = &shape.shape {
                    Some(text.galley.text())
                } else {
                    None
                }
            })
            .collect();
        assert!(
            texts.contains(&"synthetic@example.invalid"),
            "identifier is missing behind the row truncation"
        );
        assert!(texts.contains(&"Main account"));
        assert!(texts.iter().any(|text| text.starts_with("Last opened:")));
    }

    #[test]
    fn current_marker_tracks_recorded_account_instead_of_row_selection() {
        let dir = tempfile::tempdir().unwrap();
        let paths = Paths {
            data: dir.path().join("metadata"),
            config: dir.path().join("config.json"),
            launcher: dir.path().join("launcher.exe"),
        };
        let first = Account::new("first@example.invalid".into());
        let second = Account::new("second@example.invalid".into());
        let selected_id = first.id;
        paths.save(&[first, second]).unwrap();
        let ctx = egui::Context::default();
        configure_theme(&ctx);
        let mut deck = Deck::load(paths);
        deck.selected = Some(selected_id);
        for result in [
            Ok(Some("SECOND@example.invalid".into())),
            Ok(None),
            Err(anyhow::anyhow!("unreadable configuration")),
        ] {
            let expect_current = matches!(&result, Ok(Some(_)));
            let (tx, rx) = mpsc::channel();
            tx.send(result).unwrap();
            deck.current_receiver = Some(rx);
            deck.next_current_refresh = Instant::now() + Duration::from_secs(60);
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(860., 560.),
                    )),
                    ..Default::default()
                },
                |ctx| deck.render(ctx),
            );
            let mut badge_y = None;
            let mut second_y = None;
            for shape in output.shapes {
                if let egui::Shape::Text(text) = shape.shape {
                    match text.galley.text() {
                        "Current" => {
                            assert!(badge_y.is_none(), "multiple Current markers");
                            badge_y = Some(text.pos.y);
                        }
                        "second" => second_y = Some(text.pos.y),
                        _ => {}
                    }
                }
            }
            assert_eq!(deck.selected, Some(selected_id));
            assert_eq!(badge_y.is_some(), expect_current);
            if let Some(y) = badge_y {
                assert!(
                    (y - second_y.unwrap()).abs() < 10.,
                    "Current marker appeared on the selected row instead of the recorded account"
                );
            }
        }
    }

    #[test]
    fn switching_discards_stale_current_account_reads() {
        let dir = tempfile::tempdir().unwrap();
        let paths = Paths {
            data: dir.path().join("metadata"),
            config: dir.path().join("config.json"),
            launcher: dir.path().join("launcher.exe"),
        };
        let mut deck = Deck::load(paths);
        deck.current = Some("previous@example.invalid".into());
        let (tx, rx) = mpsc::channel();
        deck.current_receiver = Some(rx);
        deck.invalidate_current();
        assert!(deck.current.is_none());
        assert!(
            tx.send(Ok(Some("previous@example.invalid".into())))
                .is_err()
        );
    }

    #[test]
    fn main_actions_fit_minimum_normal_and_large_windows() {
        let dir = tempfile::tempdir().unwrap();
        let paths = Paths {
            data: dir.path().join("metadata"),
            config: dir.path().join("config.json"),
            launcher: dir.path().join("launcher.exe"),
        };
        let ctx = egui::Context::default();
        configure_theme(&ctx);
        let mut deck = Deck::load(paths);
        for populated in [false, true] {
            if populated {
                deck.accounts = vec![Account::new(format!(
                    "{}@example.invalid",
                    "long".repeat(40)
                ))];
                deck.sort();
            }
            for (width, height) in [(640., 460.), (860., 560.), (1600., 1000.)] {
                let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(width, height));
                // Render twice so panel measurements settle before checking text bounds.
                let mut output = None;
                for _ in 0..2 {
                    output = Some(ctx.run(
                        egui::RawInput {
                            screen_rect: Some(screen),
                            ..Default::default()
                        },
                        |ctx| deck.render(ctx),
                    ));
                }
                let mut found = std::collections::HashSet::new();
                for shape in output.unwrap().shapes {
                    if let egui::Shape::Text(text) = shape.shape {
                        let label = text.galley.text();
                        if ["Switch account", "Save current", "+ Add account"].contains(&label) {
                            assert!(
                                screen.expand(1.).contains_rect(egui::Rect::from_min_size(
                                    text.pos,
                                    text.galley.size()
                                )),
                                "{label} outside {width}x{height}"
                            );
                            found.insert(label.to_owned());
                        }
                    }
                }
                assert_eq!(found.len(), 3, "main action missing at {width}x{height}");
            }
        }
    }
}
