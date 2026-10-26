//! Help ▸ Check for updates (issue #28), with stand-in release sources (no network).

use std::sync::Arc;

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use pdfcraft_ui_egui::PdfCraftApp;
use pdfcraft_ui_egui::updates::{Release, UpdateSource, is_newer};

fn source(answer: Result<&str, &str>) -> UpdateSource {
    let answer = answer.map(str::to_string).map_err(str::to_string);
    Arc::new(move || answer.clone().map(|v| Release { url: format!("https://github.com/SupportMGRC/MGRC-PDF/releases/tag/{v}"), version: v }))
}

fn harness(answer: Result<&str, &str>) -> Harness<'static, PdfCraftApp> {
    Harness::builder().with_size(egui::vec2(1200.0, 800.0)).build_eframe(move |_cc| {
        let mut app = PdfCraftApp::new();
        app.set_option("language", "en").unwrap();
        app.update_source = Some(source(answer));
        app
    })
}

/// Run frames until the background check has reported. By what the dialog shows, with the
/// same 30-second deadline as `support::wait_until`, not for a fixed second: under load the
/// check's thread can take longer (#1277).
fn settle(h: &mut Harness<'static, PdfCraftApp>) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        h.run_steps(2);
        if h.query_by_label_contains("Checking for a newer version").is_none() {
            break;
        }
        assert!(std::time::Instant::now() < deadline, "timed out waiting for the update check");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    h.run_steps(2);
}

#[test]
fn versions_compare_by_number() {
    assert!(is_newer("v0.2.0", "0.1.1"));
    assert!(is_newer("v0.1.10", "0.1.9"));
    assert!(is_newer("1", "0.9.9"));
    assert!(!is_newer("v0.1.1", "0.1.1"));
    assert!(!is_newer("v0.1.0", "0.1.1"));
    assert!(!is_newer("v0.1.1-beta.2", "0.1.1"), "suffixes are ignored");
    assert!(!is_newer("nightly", "0.1.1"), "a tag that isn't a version is never newer");
    assert!(!is_newer("v1.2.3.4", "0.1.1"));
    assert!(!is_newer("v99999999999999999999.0.0", "0.1.1"), "out of range");
}

#[test]
fn a_newer_release_is_offered_for_download() {
    let mut h = harness(Ok("v99.0.0"));
    h.state_mut().execute("help.check_updates");
    settle(&mut h);
    h.get_by_label_contains("MGRC-PDF 99.0.0 is available");
    h.get_by_label("Download");
    h.get_by_label("Later").click();
    h.run_steps(3);
    assert!(h.query_by_label_contains("is available").is_none(), "Later closes the dialog");
}

#[test]
fn an_up_to_date_or_failed_check_says_so() {
    let mut h = harness(Ok(concat!("v", env!("CARGO_PKG_VERSION"))));
    h.state_mut().execute("help.check_updates");
    settle(&mut h);
    h.get_by_label_contains("is up to date");
    assert!(h.query_by_label("Download").is_none());

    let mut h = harness(Err("couldn't reach GitHub"));
    h.state_mut().execute("help.check_updates");
    settle(&mut h);
    h.get_by_label_contains("Couldn't check for updates: couldn't reach GitHub");
}

#[test]
fn nothing_is_asked_until_the_user_checks() {
    // No check at start (the owner's decision): the source is only called from the command.
    let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counted = calls.clone();
    let mut h = Harness::builder().with_size(egui::vec2(1200.0, 800.0)).build_eframe(move |_cc| {
        let mut app = PdfCraftApp::new();
        app.set_option("language", "en").unwrap();
        // Settings from a build that had the startup option are ignored.
        app.restore(r#"{"check_updates_at_start": true}"#);
        let counted = counted.clone();
        app.update_source = Some(Arc::new(move || {
            counted.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(Release { version: "v99.0.0".into(), url: "https://github.com/SupportMGRC/MGRC-PDF/releases/tag/v99.0.0".into() })
        }));
        app
    });
    settle(&mut h);
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 0);
    assert!(h.query_by_label_contains("is available").is_none());
    h.state_mut().execute("help.check_updates");
    settle(&mut h);
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    h.get_by_label_contains("MGRC-PDF 99.0.0 is available");
}

/// A harness whose app checks at start-up, the way the desktop app does.
fn startup_harness(answer: Result<&str, &str>) -> Harness<'static, PdfCraftApp> {
    Harness::builder().with_size(egui::vec2(1200.0, 800.0)).build_eframe(move |_cc| {
        let mut app = PdfCraftApp::new();
        app.set_option("language", "en").unwrap();
        app.update_source = Some(source(answer));
        app.check_updates_at_start = true;
        app
    })
}

/// Run frames until `done`, with the same 30-second deadline as `settle`.
fn wait_for(h: &mut Harness<'static, PdfCraftApp>, done: impl Fn(&PdfCraftApp) -> bool) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while !done(h.state()) {
        assert!(std::time::Instant::now() < deadline, "timed out waiting for the start-up check");
        h.run_steps(2);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    h.run_steps(2);
}

#[test]
fn a_newer_release_found_at_start_shows_the_update_bar() {
    let mut h = startup_harness(Ok("v99.0.0"));
    wait_for(&mut h, |app| app.update_available().is_some());
    h.get_by_label_contains("MGRC-PDF 99.0.0 is available");
    h.get_by_label_contains("ask IT");
    if let Ok(dir) = std::env::var("PDFCRAFT_UPDATE_SHOTS") {
        let _gpu = crate::gpu();
        h.render().unwrap().save(std::path::Path::new(&dir).join("update-bar.png")).unwrap();
    }
    h.get_by_label("Download").click();
    h.run_steps(3);
    assert_eq!(h.state().last_opened_url.as_deref(), Some("https://github.com/SupportMGRC/MGRC-PDF/releases/tag/v99.0.0"));
    assert!(h.query_by_label_contains("is available").is_none(), "Download hides the bar");
}

#[test]
fn later_hides_the_update_bar_for_this_run() {
    let mut h = startup_harness(Ok("v99.0.0"));
    wait_for(&mut h, |app| app.update_available().is_some());
    h.get_by_label("Later").click();
    h.run_steps(3);
    assert!(h.query_by_label_contains("is available").is_none());
    assert!(h.state().update_available().is_none());
    assert!(h.state().last_opened_url.is_none(), "Later opens nothing");
}

#[test]
fn an_up_to_date_or_offline_start_shows_no_bar() {
    for answer in [Ok(concat!("v", env!("CARGO_PKG_VERSION"))), Err("couldn't reach GitHub")] {
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counted = calls.clone();
        let inner = source(answer);
        let mut h = Harness::builder().with_size(egui::vec2(1200.0, 800.0)).build_eframe(move |_cc| {
            let mut app = PdfCraftApp::new();
            app.set_option("language", "en").unwrap();
            let (counted, inner) = (counted.clone(), inner.clone());
            app.update_source = Some(Arc::new(move || {
                counted.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                inner()
            }));
            app.check_updates_at_start = true;
            app
        });
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while calls.load(std::sync::atomic::Ordering::SeqCst) == 0 {
            assert!(std::time::Instant::now() < deadline, "the start-up check never ran");
            h.run_steps(2);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        h.run_steps(10);
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1, "{answer:?}: one check per launch");
        assert!(h.state().update_available().is_none(), "{answer:?}");
        assert!(h.query_by_label_contains("is available").is_none(), "{answer:?}");
        assert!(h.query_by_label_contains("Couldn't check").is_none(), "{answer:?}: the start-up check is quiet");
    }
}
