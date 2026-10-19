//! Spool a job in a separate MGRC-PDF process (Windows).
//!
//! On Windows the job is drawn by `Windows.Data.Pdf` and Direct2D into the printer driver, all in
//! native code. A driver or print support app that fails mid-job can take Direct2D down with an
//! access violation (`0xC0000005` in `d2d1.dll`, seen with the Microsoft IPP Class Driver and
//! the Ricoh print support app). No Rust error handling can catch that, so the whole app closed
//! with the user's open documents.
//!
//! [`submit_isolated`] runs the spooling in a child process instead: the app starts itself again
//! with [`WORKER_FLAG`] and a private job folder holding `job.pdf` and `job.txt`. The child
//! ([`run_worker`]) reports each sheet on standard output and its result on the last line. When
//! the child dies, the app stays open and reports what happened, with the Windows exception code.

use std::io::{BufRead as _, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};

use crate::PrintError;
use crate::spool::{Duplex, Job, Scaling};

/// The first argument that turns `selfpdf.exe` into a print worker; the job folder follows.
pub const WORKER_FLAG: &str = "--print-worker";

const JOB_PDF: &str = "job.pdf";
const JOB_TXT: &str = "job.txt";
/// The worker's exit code when spooling failed with an ordinary error.
const FAILED: i32 = 2;
/// The worker's exit code when the job folder could not be read.
const BAD_JOB: i32 = 3;

/// One line of the worker's standard output.
#[derive(Debug, PartialEq)]
enum News {
    Sent(usize, usize),
    Ok(String),
    Err(String),
}

fn parse_news(line: &str) -> Option<News> {
    let line = line.trim_end_matches(['\r', '\n']);
    if let Some(rest) = line.strip_prefix("sent ") {
        let (a, b) = rest.split_once(' ')?;
        return Some(News::Sent(a.parse().ok()?, b.parse().ok()?));
    }
    if let Some(rest) = line.strip_prefix("ok ") {
        return Some(News::Ok(unescape(rest)));
    }
    line.strip_prefix("err ").map(|rest| News::Err(unescape(rest)))
}

/// Backslash-escape so a value fits on one `key=value` line.
fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}

fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }
    out
}

/// `job.txt`: one `key=value` per line, values escaped.
fn encode_job(job: &Job) -> String {
    let mut s = String::new();
    if let Some(p) = &job.printer {
        s.push_str(&format!("printer={}\n", escape(p)));
    }
    s.push_str(&format!("copies={}\n", job.copies));
    s.push_str(&format!("collate={}\n", job.collate));
    let duplex = match job.duplex {
        None => "default",
        Some(Duplex::Off) => "off",
        Some(Duplex::LongEdge) => "long",
        Some(Duplex::ShortEdge) => "short",
    };
    s.push_str(&format!("duplex={duplex}\n"));
    let scaling = match job.scaling {
        Scaling::Exact => "exact",
        Scaling::ShrinkToFit => "shrink",
    };
    s.push_str(&format!("scaling={scaling}\n"));
    s.push_str(&format!("grayscale={}\n", job.grayscale));
    s.push_str(&format!("title={}\n", escape(&job.title)));
    for (k, v) in &job.options {
        s.push_str(&format!("option={}\t{}\n", escape(k), escape(v)));
    }
    s
}

fn decode_job(text: &str) -> Result<Job, String> {
    let mut job = Job::default();
    for line in text.lines().filter(|l| !l.is_empty()) {
        let (key, value) = line.split_once('=').ok_or_else(|| format!("bad job line {line:?}"))?;
        let bad = || format!("bad {key} value {value:?}");
        match key {
            "printer" => job.printer = Some(unescape(value)),
            "copies" => job.copies = value.parse().map_err(|_| bad())?,
            "collate" => job.collate = value.parse().map_err(|_| bad())?,
            "duplex" => {
                job.duplex = match value {
                    "default" => None,
                    "off" => Some(Duplex::Off),
                    "long" => Some(Duplex::LongEdge),
                    "short" => Some(Duplex::ShortEdge),
                    _ => return Err(bad()),
                }
            }
            "scaling" => {
                job.scaling = match value {
                    "exact" => Scaling::Exact,
                    "shrink" => Scaling::ShrinkToFit,
                    _ => return Err(bad()),
                }
            }
            "grayscale" => job.grayscale = value.parse().map_err(|_| bad())?,
            "title" => job.title = unescape(value),
            // Escaped values hold no raw tab, so the first tab separates key and value.
            "option" => {
                let (k, v) = value.split_once('\t').ok_or_else(bad)?;
                job.options.push((unescape(k), unescape(v)));
            }
            _ => return Err(format!("unknown job key {key:?}")),
        }
    }
    Ok(job)
}

/// A private folder for one job under the temporary directory.
fn job_dir() -> std::io::Result<PathBuf> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static N: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.subsec_nanos()).unwrap_or(0);
    let base = std::env::temp_dir();
    let mut last = None;
    for _ in 0..16 {
        let dir = base.join(format!("mgrc-pdf-print-{}-{nanos:x}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
        match std::fs::create_dir(&dir) {
            Ok(()) => return Ok(dir),
            Err(e) => last = Some(e),
        }
    }
    Err(last.unwrap_or_else(|| std::io::Error::other("no job folder")))
}

/// An exit code as Windows reports an unhandled exception: its NTSTATUS, named when known.
fn describe_code(code: i32) -> String {
    // Bit-for-bit reinterpretation: NTSTATUS values are unsigned with the top bit set.
    let code = code as u32;
    let name = match code {
        0xC000_0005 => " (access violation)",
        0xC000_0409 => " (fast fail)",
        0xC000_00FD => " (stack overflow)",
        _ => "",
    };
    format!("0x{code:08X}{name}")
}

/// What to tell the user when the worker ended without a result line.
fn died_message(status: ExitStatus, sent: Option<(usize, usize)>) -> String {
    let code = status.code().map(describe_code);
    #[cfg(unix)]
    let code = code.or_else(|| std::os::unix::process::ExitStatusExt::signal(&status).map(|s| format!("signal {s}")));
    let code = code.unwrap_or_else(|| "unknown".into());
    let progress = match sent {
        Some((sent, sheets)) if sheets > 0 => format!(" after {sent} of {sheets} sheet(s) were sent"),
        _ => String::new(),
    };
    format!(
        "the printer driver stopped working{progress} (Windows error {code}). MGRC-PDF is still open and your documents are safe. \
         Cancel the job in the printer's queue, then try again. If it keeps happening, install the printer maker's own driver \
         (the Microsoft IPP Class Driver is known to fail on some printers)"
    )
}

/// Spool through `worker` (the app itself with [`WORKER_FLAG`] and the job folder appended),
/// relaying its progress. Never panics, and survives the worker crashing.
pub fn submit_via_worker(mut worker: Command, pdf: &[u8], job: &Job, progress: &mut dyn FnMut(usize, usize)) -> Result<String, PrintError> {
    let dir = job_dir().map_err(|e| PrintError::Spool(format!("the print job could not be written: {e}")))?;
    let result = run_in(&mut worker, &dir, pdf, job, progress);
    // Best effort: the temporary folder is cleaned by the system otherwise.
    let _ = std::fs::remove_dir_all(&dir);
    result
}

fn run_in(worker: &mut Command, dir: &Path, pdf: &[u8], job: &Job, progress: &mut dyn FnMut(usize, usize)) -> Result<String, PrintError> {
    let written = |e: std::io::Error| PrintError::Spool(format!("the print job could not be written: {e}"));
    std::fs::write(dir.join(JOB_PDF), pdf).map_err(written)?;
    std::fs::write(dir.join(JOB_TXT), encode_job(job)).map_err(written)?;
    worker.arg(WORKER_FLAG).arg(dir).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null());
    #[cfg(windows)]
    {
        // A GUI program; no console window for the worker.
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        std::os::windows::process::CommandExt::creation_flags(worker, CREATE_NO_WINDOW);
    }
    let mut child = worker.spawn().map_err(|e| PrintError::Spool(format!("the print job could not be started: {e}")))?;
    let mut sent = None;
    let mut outcome = None;
    if let Some(out) = child.stdout.take() {
        for line in BufReader::new(out).lines() {
            let Ok(line) = line else { break };
            match parse_news(&line) {
                Some(News::Sent(s, n)) => {
                    sent = Some((s, n));
                    progress(s, n);
                }
                Some(News::Ok(msg)) => outcome = Some(Ok(msg)),
                Some(News::Err(msg)) => outcome = Some(Err(msg)),
                None => {}
            }
        }
    }
    let status = child.wait().map_err(|e| PrintError::Spool(format!("the print job stopped unexpectedly: {e}")))?;
    match outcome {
        Some(Ok(msg)) if status.success() => Ok(msg),
        Some(Err(msg)) => Err(PrintError::Spool(msg)),
        // A result line followed by a crash, or no result at all.
        _ => Err(PrintError::Spool(died_message(status, sent))),
    }
}

/// How the worker spools: [`crate::spool::submit_with_progress`], or a stand-in in tests.
pub type SpoolFn<'a> = dyn FnMut(&[u8], &Job, &mut dyn FnMut(usize, usize)) -> Result<String, PrintError> + 'a;

/// The worker's side: spool the job in `dir` with `spool`, writing progress and the result to
/// `out`. Returns the process exit code.
pub fn run_worker_with(dir: &Path, out: &mut dyn std::io::Write, spool: &mut SpoolFn<'_>) -> i32 {
    let job = std::fs::read_to_string(dir.join(JOB_TXT)).map_err(|e| e.to_string()).and_then(|t| decode_job(&t));
    let pdf = std::fs::read(dir.join(JOB_PDF)).map_err(|e| e.to_string());
    let (job, pdf) = match (job, pdf) {
        (Ok(j), Ok(p)) => (j, p),
        (Err(e), _) | (_, Err(e)) => {
            let _ = writeln!(out, "err {}", escape(&format!("the print job could not be read: {e}")));
            return BAD_JOB;
        }
    };
    let result = spool(&pdf, &job, &mut |s, n| {
        let _ = writeln!(out, "sent {s} {n}");
        let _ = out.flush();
    });
    let code = match &result {
        Ok(msg) => {
            let _ = writeln!(out, "ok {}", escape(msg));
            0
        }
        Err(e) => {
            let _ = writeln!(out, "err {}", escape(&e.to_string()));
            FAILED
        }
    };
    let _ = out.flush();
    code
}

/// The worker's entry point: `selfpdf.exe --print-worker <dir>` calls this before anything else
/// and exits with its result.
pub fn run_worker(dir: &Path) -> i32 {
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    run_worker_with(dir, &mut out, &mut |pdf, job, progress| crate::spool::submit_with_progress(pdf, job, progress))
}

/// [`crate::spool::submit_with_progress`], in a separate process on Windows (see the module
/// documentation) so a crash in the printer driver can't close the app. Elsewhere, and when the
/// program's own path is unknown, the job is spooled in this process.
pub fn submit_isolated(pdf: &[u8], job: &Job, progress: &mut dyn FnMut(usize, usize)) -> Result<String, PrintError> {
    if cfg!(windows)
        && let Ok(exe) = std::env::current_exe()
    {
        return submit_via_worker(Command::new(exe), pdf, job, progress);
    }
    crate::spool::submit_with_progress(pdf, job, progress)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    fn sample_job() -> Job {
        Job {
            printer: Some("RICOH IM C2010 [583879B6E6A9]".into()),
            copies: 3,
            collate: false,
            duplex: Some(Duplex::LongEdge),
            scaling: Scaling::ShrinkToFit,
            grayscale: true,
            title: "Asset\tHandover\\Form\nline=2".into(),
            options: vec![("InputSlot".into(), "Tray 2".into()), ("a\tb".into(), "c=d\\".into())],
        }
    }

    #[test]
    fn a_job_survives_the_job_file() {
        let job = sample_job();
        assert_eq!(decode_job(&encode_job(&job)), Ok(job));
        let default = Job::default();
        assert_eq!(decode_job(&encode_job(&default)), Ok(default));
    }

    #[test]
    fn a_bad_job_file_is_an_error_not_a_panic() {
        assert!(decode_job("copies=lots").is_err());
        assert!(decode_job("nonsense").is_err());
        assert!(decode_job("duplex=sideways").is_err());
        assert!(decode_job("option=no-tab").is_err());
    }

    #[test]
    fn news_lines_parse() {
        assert_eq!(parse_news("sent 1 3\r\n"), Some(News::Sent(1, 3)));
        assert_eq!(parse_news("ok spooled 3 sheet(s)"), Some(News::Ok("spooled 3 sheet(s)".into())));
        assert_eq!(parse_news("err a\\nb"), Some(News::Err("a\nb".into())));
        assert_eq!(parse_news("sent x"), None);
        assert_eq!(parse_news("garbage"), None);
    }

    fn sh(script: &str) -> Command {
        let mut c = Command::new("sh");
        // `$0` takes WORKER_FLAG and `$1` the job folder.
        c.arg("-c").arg(script);
        c
    }

    /// The regression test for the RICOH crash: the worker dies the way selfpdf.exe died (killed
    /// by a fault mid-job) and the caller gets an error instead of going down with it.
    #[test]
    fn a_worker_that_crashes_mid_job_is_reported_not_fatal() {
        let mut seen = Vec::new();
        let result = submit_via_worker(sh("echo 'sent 0 3'; kill -SEGV $$"), b"%PDF-1.7", &Job::default(), &mut |s, n| seen.push((s, n)));
        let Err(PrintError::Spool(msg)) = result else { panic!("expected an error, got {result:?}") };
        assert!(msg.contains("stopped working"), "{msg}");
        assert!(msg.contains("after 0 of 3 sheet(s)"), "{msg}");
        assert!(msg.contains("signal 11"), "{msg}");
        assert_eq!(seen, [(0, 3)]);
    }

    #[test]
    fn a_windows_exception_code_is_named() {
        assert_eq!(describe_code(0xC000_0005_u32 as i32), "0xC0000005 (access violation)");
        assert_eq!(describe_code(1), "0x00000001");
    }

    #[test]
    fn a_worker_failure_passes_its_message_through() {
        let result = submit_via_worker(sh("echo 'err the printer is offline'; exit 2"), b"%PDF", &Job::default(), &mut |_, _| {});
        assert_eq!(result, Err(PrintError::Spool("the printer is offline".into())));
    }

    #[test]
    fn a_worker_that_succeeds_relays_progress_and_result() {
        let mut seen = Vec::new();
        let script =
            "test -f \"$1/job.pdf\" && grep -q '^copies=1$' \"$1/job.txt\" && printf 'sent 0 2\\nsent 1 2\\nsent 2 2\\nok spooled 2 sheet(s)\\n'";
        let result = submit_via_worker(sh(script), b"%PDF", &Job::default(), &mut |s, n| seen.push((s, n)));
        assert_eq!(result, Ok("spooled 2 sheet(s)".into()));
        assert_eq!(seen, [(0, 2), (1, 2), (2, 2)]);
    }

    #[test]
    fn the_worker_side_reads_the_job_and_reports() {
        let dir = job_dir().unwrap();
        std::fs::write(dir.join(JOB_PDF), b"%PDF-x").unwrap();
        std::fs::write(dir.join(JOB_TXT), encode_job(&sample_job())).unwrap();
        let mut out = Vec::new();
        let code = run_worker_with(&dir, &mut out, &mut |pdf, job, progress| {
            assert_eq!(pdf, b"%PDF-x");
            assert_eq!(job, &sample_job());
            progress(0, 1);
            progress(1, 1);
            Err(PrintError::Spool("driver said\nno".into()))
        });
        assert_eq!(code, FAILED);
        assert_eq!(String::from_utf8(out).unwrap(), "sent 0 1\nsent 1 1\nerr driver said\\nno\n");

        std::fs::remove_file(dir.join(JOB_TXT)).unwrap();
        let mut out = Vec::new();
        let code = run_worker_with(&dir, &mut out, &mut |_, _, _| Ok(String::new()));
        assert_eq!(code, BAD_JOB);
        assert!(String::from_utf8(out).unwrap().starts_with("err the print job could not be read"));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
