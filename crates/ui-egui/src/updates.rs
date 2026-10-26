//! Help ▸ Check for updates (issue #28): ask for the latest release and offer its download page.
//!
//! The desktop app supplies how to ask ([`PdfCraftApp::update_source`]), so this crate has no
//! network code; without a source (the web build, tests) the command opens the releases page.
//! MGRC-PDF never downloads or installs anything itself: the user downloads the new version.
//!
//! The desktop app also asks once, quietly, at start-up ([`PdfCraftApp::check_updates_at_start`]):
//! when a newer release exists, a bar under the mode bar offers its download page. Nothing shows
//! when MGRC-PDF is up to date or GitHub can't be reached. Installing needs an administrator on
//! MGRC's PCs, so the bar says to ask IT if Windows asks for a password.

use std::sync::Arc;

use egui::{Align, Layout};

use crate::{PdfCraftApp, theme, widgets};

/// Where every MGRC-PDF release is listed.
pub const RELEASES_PAGE: &str = "https://github.com/SupportMGRC/MGRC-PDF/releases";

/// The latest published release.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Release {
    /// Its version tag, such as `v0.2.0`.
    pub version: String,
    /// Its page on [`RELEASES_PAGE`], where the downloads are.
    pub url: String,
}

/// Asks for the latest release (blocking; it runs on its own thread).
pub type UpdateSource = Arc<dyn Fn() -> Result<Release, String> + Send + Sync>;

/// Whether release `latest` (a tag such as `v0.2.0`) is newer than version `current` (`0.1.1`).
/// Pre-release and build suffixes are ignored; a version that doesn't parse is never newer.
pub fn is_newer(latest: &str, current: &str) -> bool {
    matches!((parse(latest), parse(current)), (Some(l), Some(c)) if l > c)
}

fn parse(v: &str) -> Option<(u64, u64, u64)> {
    let v = v.trim().trim_start_matches(['v', 'V']);
    let core = v.split(['-', '+']).next()?;
    let mut parts = core.split('.');
    let mut next = |required: bool| match parts.next() {
        Some(p) => p.parse::<u64>().ok(),
        None if required => None,
        None => Some(0),
    };
    let version = (next(true)?, next(false)?, next(false)?);
    parts.next().is_none().then_some(version)
}

/// Where a check is.
#[derive(Default)]
pub(crate) enum Check {
    #[default]
    Idle,
    #[cfg(not(target_arch = "wasm32"))]
    Running(std::sync::mpsc::Receiver<Result<Release, String>>),
    Done(Result<Release, String>),
}

#[derive(Default)]
pub(crate) struct Updates {
    pub(crate) check: Check,
    /// The Updates dialog is showing.
    pub(crate) open: bool,
    /// The quiet start-up check, kept apart from the dialog's so neither shows the other's result.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub(crate) startup: Check,
    /// The start-up check has been started (it runs once per launch).
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub(crate) startup_started: bool,
    /// A newer release the start-up check found; the update bar shows it until Download or Later.
    pub(crate) available: Option<Release>,
}

impl PdfCraftApp {
    /// Help ▸ Check for updates: ask for the latest release and show the outcome.
    pub fn check_for_updates(&mut self) {
        let Some(source) = self.update_source.clone() else {
            self.open_url(RELEASES_PAGE);
            return;
        };
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.updates.open = true;
            if matches!(self.updates.check, Check::Running(_)) {
                return;
            }
            self.updates.check = Check::Running(spawn_check(source, self.ctx.clone()));
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = source;
            self.open_url(RELEASES_PAGE);
        }
    }

    /// Pick up a finished check (each frame), and start the start-up check on the first frame.
    pub(crate) fn poll_updates(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if self.check_updates_at_start && !self.updates.startup_started {
                self.updates.startup_started = true;
                if let Some(source) = self.update_source.clone() {
                    self.updates.startup = Check::Running(spawn_check(source, self.ctx.clone()));
                }
            }
            if let Some(result) = take_finished(&mut self.updates.check) {
                self.updates.check = Check::Done(result);
            }
            if let Some(result) = take_finished(&mut self.updates.startup) {
                // Quiet: only a newer release is worth a bar; up to date or offline shows nothing.
                if let Ok(release) = result
                    && is_newer(&release.version, env!("CARGO_PKG_VERSION"))
                {
                    self.updates.available = Some(release);
                }
            }
        }
    }
}

/// Run `source` on its own thread; the receiver gets its answer, and `ctx` is woken to show it.
#[cfg(not(target_arch = "wasm32"))]
fn spawn_check(source: UpdateSource, ctx: Option<egui::Context>) -> std::sync::mpsc::Receiver<Result<Release, String>> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        // The receiver may be gone (the app quit): nothing to report to then.
        let _ = tx.send(source());
        if let Some(ctx) = ctx {
            ctx.request_repaint();
        }
    });
    rx
}

/// The answer of a running check once it has one (the check then goes back to idle).
#[cfg(not(target_arch = "wasm32"))]
fn take_finished(check: &mut Check) -> Option<Result<Release, String>> {
    let Check::Running(rx) = check else { return None };
    let result = match rx.try_recv() {
        Ok(r) => r,
        Err(std::sync::mpsc::TryRecvError::Empty) => return None,
        Err(std::sync::mpsc::TryRecvError::Disconnected) => Err("the update check stopped unexpectedly".into()),
    };
    *check = Check::Idle;
    Some(result)
}

/// The update bar under the mode bar: shown when the start-up check found a newer release.
/// Download opens that release's page; Later hides the bar until the next launch.
pub(crate) fn bar(app: &mut PdfCraftApp, ui: &mut egui::Ui) {
    let Some(release) = app.updates.available.clone() else { return };
    let t = theme::Tokens::get(ui.ctx());
    let version = release.version.trim_start_matches(['v', 'V']);
    let text = crate::i18n::fmt(
        tl!("MGRC-PDF {v} is available. Download it from the release page; if Windows asks for an administrator password, ask IT to enter it."),
        &[("v", version)],
    );
    let (mut download, mut later) = (false, false);
    egui::Panel::top("update_bar").frame(egui::Frame::NONE.fill(t.accent_soft).inner_margin(egui::Margin::symmetric(14, 7))).show(ui, |ui| {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            later = widgets::pill_button(ui, tl!("Later"), false).clicked();
            download = widgets::pill_button(ui, tl!("Download"), true).clicked();
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                ui.add(crate::icons::image("cloud", 16.0, t.accent_text));
                let label = egui::Label::new(egui::RichText::new(text).color(t.text));
                ui.add(if ui.available_width() >= 120.0 { label.wrap() } else { label.truncate() });
            });
        });
    });
    if download || later {
        app.updates.available = None;
    }
    if download {
        app.open_url(&release.url);
    }
}

impl PdfCraftApp {
    /// The release the start-up check found newer than this version, while the bar shows it.
    pub fn update_available(&self) -> Option<&Release> {
        self.updates.available.as_ref()
    }
}

/// The Updates dialog.
pub(crate) fn dialog(app: &mut PdfCraftApp, ctx: &egui::Context) {
    if !app.updates.open {
        return;
    }
    let t = theme::Tokens::get(ctx);
    let current = env!("CARGO_PKG_VERSION");
    let mut close = false;
    let mut download: Option<String> = None;
    let modal = crate::widgets::show_modal(ctx, "updates", |ui| {
        ui.set_width(420.0);
        ui.horizontal(|ui| {
            ui.add(crate::icons::image("cloud", 22.0, t.accent));
            ui.label(egui::RichText::new(tl!("Check for updates")).font(theme::semibold(16.0)));
        });
        ui.add_space(8.0);
        match &app.updates.check {
            Check::Idle => {
                ui.label(tl!("No check has run yet."));
            }
            #[cfg(not(target_arch = "wasm32"))]
            Check::Running(_) => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(tl!("Checking for a newer version…"));
                });
            }
            Check::Done(Ok(r)) if is_newer(&r.version, current) => {
                let version = r.version.trim_start_matches(['v', 'V']);
                ui.label(egui::RichText::new(crate::i18n::fmt(tl!("MGRC-PDF {v} is available."), &[("v", version)])).strong());
                ui.label(
                    egui::RichText::new(crate::i18n::fmt(
                        tl!("You have version {c}. Download the new version from its release page."),
                        &[("c", current)],
                    ))
                    .color(t.text_muted),
                );
                download = Some(r.url.clone());
            }
            Check::Done(Ok(_)) => {
                ui.label(crate::i18n::fmt(tl!("MGRC-PDF {c} is up to date."), &[("c", current)]));
            }
            Check::Done(Err(e)) => {
                ui.label(crate::i18n::fmt(tl!("Couldn't check for updates: {e}"), &[("e", &e.to_string())]));
                ui.label(
                    egui::RichText::new(crate::i18n::fmt(
                        tl!("You have version {c}. All releases are listed at {page}."),
                        &[("c", current), ("page", RELEASES_PAGE)],
                    ))
                    .color(t.text_muted),
                );
            }
        }
        ui.add_space(10.0);
        ui.label(
            egui::RichText::new(tl!("Asks GitHub for the latest release. Nothing is downloaded or installed automatically."))
                .color(t.text_muted)
                .small(),
        );
        ui.add_space(12.0);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if let Some(url) = download.take() {
                let get = widgets::pill_button(ui, tl!("Download"), true).clicked();
                let later = widgets::pill_button(ui, tl!("Later"), false).clicked();
                close = get || later;
                download = get.then_some(url);
            } else if widgets::pill_button(ui, tl!("Close"), true).clicked() {
                close = true;
            }
        });
    });
    if modal.should_close() {
        close = true;
        download = None;
    }
    if close {
        app.updates.open = false;
        if let Some(url) = download {
            app.open_url(&url);
        }
    }
}
