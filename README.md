<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/brand/mgrc-logo-white.svg">
    <img alt="Malaysian Genomics Resource Centre" src="docs/brand/mgrc-logo.svg" width="320">
  </picture>
</p>

<h1 align="center">MGRC-PDF</h1>

<p align="center">
  <b>The PDF workbench of Malaysian Genomics Resource Centre Berhad (MGRC).</b><br>
  Read, organize, combine, split, comment on, fill in, sign and protect PDFs in a fast, native app.<br>
  Works offline. No account, no telemetry, no cloud.
</p>

<p align="center">
  <a href="#install">Install</a> ·
  <a href="#what-it-does">What it does</a> ·
  <a href="#using-it">Using it</a> ·
  <a href="#command-line-tool">Command-line tool</a> ·
  <a href="#building-from-source">Building</a> ·
  <a href="#releasing">Releasing</a> ·
  <a href="#support">Support</a> ·
  <a href="#licence">Licence</a>
</p>

---

## About

MGRC-PDF is MGRC's in-house PDF application for staff. It runs entirely on the user's own
computer: documents are never uploaded anywhere, and nothing is sent back to MGRC or anyone else.

MGRC-PDF is open source based project.

| | |
|---|---|
| **Program** | `selfpdf.exe` (Windows), `selfpdf` (macOS, Linux) |
| **Command-line tool** | `selfpdf-cli` |
| **App ID** | `selfpdf` |
| **Publisher** | Malaysian Genomics Resource Centre Berhad |
| **Maintainer** | Noor, Software Engineering & System Support |

## Install

### Windows (staff PCs)

Use the installer `selfpdf-<version>-windows-x64.msi`.

- **Pushed by IT (Intune or Group Policy):** installs silently for all users:

  ```bat
  msiexec /i selfpdf-<version>-windows-x64.msi /qn ALLUSERS=1
  ```

- **Installed by hand:** double-click the MSI. Until the installer is code-signed, Windows may show
  *"Windows protected your PC"*; choose **More info → Run anyway**.

The installer adds MGRC-PDF to the Start menu and the desktop, offers it under **Open with** for PDF
files, and adds **Edit with MGRC-PDF** to the right-click menu of PDFs. It never makes itself the
default PDF reader; each user can choose it in **Settings → Apps → Default apps**.

**No installation rights?** Use `selfpdf-<version>-windows-x64-portable.zip`: unzip it anywhere and
run `selfpdf.exe`. Settings stay in a folder next to the program.

### Uninstall

**Settings → Apps → Installed apps → MGRC-PDF → Uninstall**, or
`msiexec /x selfpdf-<version>-windows-x64.msi /qn`.

## What it does

**Read**
- Fast, faithful rendering, including complex scripts, vertical Japanese, colour emoji,
  transparency and gradients.
- Single page, continuous, two-page and full-screen views; zoom, rotate, dark mode.
- Search, select and copy text in reading order; bookmarks, page thumbnails and page labels.

**Organize**
- Rotate, delete, move, insert and extract pages in a page grid.
- Combine several PDFs into one, or split one into several.

**Review and fill in**
- Comments: sticky notes, highlights, underline, strike-through, shapes, ink and text boxes, with
  replies and status.
- Fill in and clear form fields; prepare forms by detecting fields.
- Fill & Sign, stamps and digital signatures with certificates.

**Edit and create**
- Edit existing text and images, add text and images.
- Create PDFs from images, scan and recognise text (OCR), compress, and export pages as images.

**Protect**
- Open password-protected PDFs and respect their permissions.
- Add passwords, redact content permanently and remove hidden information.

**Safe with your work**
- Saving appends your changes and keeps the original bytes intact; writes are atomic.
- Deep undo and redo, and crash recovery if the app closes unexpectedly.

## Using it

- **Home screen:** recommended tools, recent files and pinned folders.
- **All tools** (left panel): every tool by task.
- **Find tools and commands** (top right, `Ctrl+K`): type what you want to do.
- **Light or dark:** click the sun / moon icon at the top right to cycle System → Light → Dark, or
  right-click it to choose. *System* follows Windows.
- **Interface language:** **Menu → Edit → Preferences… → Interface language** (`Ctrl+,`).
- **Keyboard shortcuts:** the **?** icon at the top right.

Where MGRC-PDF keeps its files on Windows:

| What | Where |
|---|---|
| Settings, logs, crash recovery | `%APPDATA%\MGRC-PDF\` |
| Log file (attach to support requests) | `%APPDATA%\MGRC-PDF\data\logs\pdfcraft.log` |
| Portable edition | `SelfPdfData\` next to `selfpdf.exe` |

## Command-line tool

`selfpdf-cli` uses the same engine as the app, for scripting and batch jobs:

```sh
selfpdf-cli info   form.pdf                               # structure as JSON
selfpdf-cli text   report.pdf --page 3                    # text in reading order
selfpdf-cli render report.pdf --page 1 --dpi 150 --out p1.png
selfpdf-cli edit   in.pdf --rotate 1,2:90 --delete 5 --title "Q3" --out out.pdf
selfpdf-cli combine a.pdf b.pdf --out combined.pdf
selfpdf-cli split  big.pdf --every 10 --out-dir parts
selfpdf-cli batch  scans/ --steps ocr,optimize --out-dir done
selfpdf-cli --help                                        # everything else
```

For automation and AI agents, `selfpdf-cli run` and the opt-in MCP server (`selfpdf-cli mcp`)
expose every feature as a JSON tool; see [docs/mcp.md](docs/mcp.md) and
[docs/cli-batch.md](docs/cli-batch.md). Neither runs unless you start it.

## Building from source

Needs Rust 1.95 or later (`rustup`, MSVC toolchain on Windows).

```sh
git clone https://github.com/SupportMGRC/MGRC-PDF.git
cd MGRC-PDF
cargo xtask models                          # once: OCR models used by Scan & OCR
```

### Test while coding (like `localhost`)

```sh
cargo run -p pdfcraft                       # build a test version and open the app
cargo run -p pdfcraft -- some.pdf           # same, opening a PDF
cargo run -p selfpdf-cli -- --help          # the command-line tool
```

Quick to build, a little slower to run. Use it to check your changes while you work. There is no
auto-reload: close the app and run the command again after each change.

### Build what users get

```sh
cargo build --release -p pdfcraft
```

Builds the fast, final version into `target\release\selfpdf.exe`, the same kind of program the
installer ships. Slower to build. Use it to test exactly what staff will see before a release. This
only builds the program on your PC; the installer that staff download is made as described in
[Releasing](#releasing).

### Before committing

Run the checks in [CLAUDE.md](CLAUDE.md#quality-checks).

### Code layout

The code is a Cargo workspace. The engine (parsing, rendering, editing) lives in library crates under
`crates/`; the desktop interface is `crates/ui-egui`; the two programs, `selfpdf` and `selfpdf-cli`,
are under `apps/`. See [docs/architecture.md](docs/architecture.md).

## Releasing

Windows installers are built by GitHub Actions (**Actions → Release (MGRC Windows) → Run
workflow**, or push a `v*` tag). Step-by-step: [docs/releasing.md](docs/releasing.md).

Branches: work is committed on `Noor`, then merged into `main`. `main` is what gets released.

## Support

MGRC staff: contact **Software Engineering & System Support (IT)**. Please include what you were
doing, the PDF if it is not confidential, and the log file listed under [Using it](#using-it).

## Licence

The source code is licensed under the MIT License or the Apache License 2.0, at your option
([LICENSE-MIT](LICENSE-MIT), [LICENSE-APACHE](LICENSE-APACHE)). Required notices are in
[NOTICE](NOTICE), and every bundled icon, font and image is listed with its licence in
[ATTRIBUTION.md](ATTRIBUTION.md).

The MGRC name, logos and app icon are trademarks of Malaysian Genomics Resource Centre Berhad and
are not covered by the code licence; see [docs/brand/LICENSE-brand.txt](docs/brand/LICENSE-brand.txt).

<br>

<p align="center">
  <img alt="MGRC" src="docs/brand/mgrc-mark.svg" width="28"><br>
  <sub><b>Developed and maintained by Noor</b><br>
  © Malaysian Genomics Resource Centre Berhad</sub>
</p>
