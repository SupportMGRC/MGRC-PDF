# MGRC-PDF — instructions for AI coding agents

MGRC-PDF is the PDF application of Malaysian Genomics Resource Centre Berhad (MGRC), written in
Rust. It is maintained by Noor (Software Engineering & System Support). The desktop program is
`selfpdf` (`apps/pdfcraft`), the command-line tool is `selfpdf-cli` (`apps/pdfcraft-cli`), and the
app ID is `selfpdf`. Windows is the primary target for MGRC staff.

`AGENTS.md` holds the rules that bind every change. If this file and `AGENTS.md` disagree,
`AGENTS.md` wins.

## Start every session here
1. Read `README.md` for what the app is and how it ships, and the `README.md` of any crate you are
   about to touch.
2. Make the change on the `Noor` branch. Never commit to `main` directly; `main` only receives
   merges from `Noor` (see [Branches](#branches)).
3. Run the [quality checks](#quality-checks) for the crates you touched before you commit.

## Non-negotiables
- **Assets.** Read `AGENTS.md` §1 before adding or showing any icon, image, font or document. Only
  openly licensed or MGRC-owned assets, each with an entry in `ATTRIBUTION.toml`. Never add assets
  from Adobe products. `cargo xtask assets` enforces this.
- **MGRC branding.** The MGRC logos live in `docs/brand/mgrc-*.svg` and the app icons in
  `assets/app-icon/` (regenerate them with `packaging/icons.sh`). The visible name is **MGRC-PDF**;
  file and program names stay lowercase (`selfpdf`, `selfpdf-cli`). The credit line
  "Developed and maintained by Noor" (`widgets::CREDIT`) stays on the home screen and in About.
- **No outside services.** The app works offline. Don't add telemetry, accounts, cloud calls or
  community links. Any new network access needs Noor's approval.
- **Clean-room.** Behaviour comes from public specifications (ISO 32000-2) and documentation. Never
  copy GPL/AGPL code, and never read or copy anything from inside Adobe products.
- **Layering.** The engine crates never depend on the UI (egui, winit, eframe, rfd). `cos`,
  `filters` and `crypt` stay standalone. `cargo xtask layers` checks this.
- **Fidelity.** The PDF object graph is the model. Preserve unknown data. Saves are incremental
  unless a full rewrite is required. Never silently drop or repair data without recording it.
- **Never panic.** Every PDF, script, settings file and keystroke is untrusted input. Return
  `Result` and propagate with `?`. In non-test code:
  - no `unwrap`/`expect`/`panic!`/`unreachable!`/`todo!` unless provably infallible, with a comment;
  - no indexing or slicing with input-derived positions (use `get`; slice strings at char boundaries);
  - checked or saturating arithmetic on input-derived numbers; guard division by zero and NaN/inf;
  - cap allocations sized by input; bound recursion;
  - no `unsafe` (`unsafe_code = "forbid"`).
  Every crash fix gets a small synthetic regression test (see `AGENTS.md` §4).
- **Rust only** in the product and build tooling (`xtask`).

## Quality checks
Before every commit, scoped to the crates you touched (the full workspace is slow; CI runs it):

```sh
cargo fmt --all --check
cargo clippy -p <crate> -j 2 -- -D warnings
cargo test   -p <crate> -j 2
cargo xtask layers
cargo xtask assets          # after changing any asset: cargo xtask assets --write first
```

Prove a new test fails before the fix by reverting the fix once.

## Running and looking at the app
- `cargo run -p pdfcraft -- <file.pdf>` opens the desktop app (`selfpdf`).
- `cargo run -p selfpdf-cli -- run --script steps.json --root DIR` drives the engine headlessly
  (`selfpdf-cli tools` lists the tools). `selfpdf-cli mcp` is the opt-in MCP server.
- For UI changes, **look at the result**: start `selfpdf --control FILE doc.pdf`, then use
  `selfpdf-cli ui --control FILE screenshot --out x.png` (also `inspect`, `click`, `key`, `type`,
  `command id=view.theme.dark`).
- `cargo xtask fuzz --time 300` fuzzes open/render/edit/save; turn every finding into a regression
  test before fixing it.

## Branches
- `Noor`: all work is committed and pushed here first.
- `main`: release branch. Update it only by merging `Noor`:

  ```sh
  git checkout main && git merge Noor && git push
  ```

Releases are built from `main`; see `docs/releasing.md`.
