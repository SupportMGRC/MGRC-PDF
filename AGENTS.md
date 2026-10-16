# AGENTS.md — rules for every change to MGRC-PDF

These rules bind every person and every AI agent working in this repository, before any task,
instruction or deadline. `CLAUDE.md` holds the day-to-day working instructions. If the two
disagree, **this file wins.** Decisions this file reserves for "the maintainer" belong to Noor.

---

## 1. Asset policy (no exceptions)

An **asset** is any non-code material the project includes, ships, embeds, generates for
publication, or shows in its documentation: icons, images, logos, screenshots, fonts, colour
profiles and other binary data, sounds, video, and sample or test documents.

### 1.1 Nothing from Adobe products
- **Never** use icons, artwork, logos, UI graphics, fonts, stamps, cursors, sample documents or any
  other asset taken from an Adobe product, installer, website or package. This includes fonts that
  Adobe products install and Adobe-designed open fonts (Source Han, Source Sans/Serif, and Noto CJK,
  which is Source Han rebranded).
- **Never** recreate, trace or closely imitate an Adobe icon or graphic. Icons come from Lucide.
- Screenshots of Adobe software are never committed, published or used as a design source.
- The only Adobe-authored material allowed is **non-visual technical data** required for PDF
  compatibility, each item recorded in `ATTRIBUTION.toml` with `adobe_data = true`. The list is
  closed:
  - the CMap resources (BSD-3-Clause) inside the `hayro-cmap` dependency;
  - the standard-14 font metrics and encoding tables from the PDF specification, in the hayro
    dependency and in `crates/fonts/src/std14_metrics.rs`.

  Adding anything to this list needs the maintainer's approval.

### 1.2 Only open, provable licences
Every asset must be one of:
- **Open source:** ISC, MIT, BSD-2-Clause, BSD-3-Clause, Apache-2.0, OFL-1.1, Ubuntu-Font-1.0 or
  Bitstream-Vera.
- **Creative Commons:** CC0-1.0, CC-BY-4.0 or CC-BY-SA-4.0.
- **Public domain.**
- **Original work** made for MGRC-PDF and licensed under the project licence (`MIT OR Apache-2.0`).

**The MGRC brand is the one exception.** The MGRC name, logos (`docs/brand/mgrc-*.svg`) and the
app icons rendered from the MGRC mark (`assets/app-icon/`) are trademarks of Malaysian Genomics
Resource Centre Berhad, not open source (manifest identifier `LicenseRef-MGRC-Trademark`, terms in
`docs/brand/LICENSE-brand.txt`). Use them unmodified, and only to identify MGRC-PDF.

Anything else is forbidden, including anything whose licence you cannot prove: system fonts, emoji
artwork from Apple/Microsoft/Samsung, stock photos, "free for personal use" material and
screenshots of other software. Screenshots of MGRC-PDF are allowed when everything shown in them is
allowed.

### 1.3 Every asset is attributed
- **`ATTRIBUTION.toml`** is the single source of truth: one entry per asset file (committed,
  fetched at build time or bundled by a dependency) with path or URL, title, author, source, SPDX
  licence, licence file, SHA-256 and usage.
- **`ATTRIBUTION.md`** is generated from it with `cargo xtask assets --write`. Never edit it by hand.
- **`NOTICE`** keeps the notices the licences require.
- **`cargo xtask assets`** fails when an asset has no entry, a hash does not match, a licence is not
  allowed, a licence file is missing, or an image, icon or font names Adobe as its source.

### 1.4 Fonts
- Don't commit new font files. The Latin interface fonts in `assets/fonts/` (Inter, JetBrains Mono,
  Dancing Script) stay.
- Japanese and Chinese faces come from an optional font pack passed at build time as
  `CRAFT_FONTS_DIR`. Without it MGRC-PDF builds and runs normally; only Japanese/Chinese interface
  text and writing those scripts into PDFs are unavailable. MGRC's Windows release does not need it.
- The desktop app may read one font already installed on the user's computer as a last fallback for
  missing characters. It is never embedded, shipped or committed, and nothing from it enters a PDF.

### 1.5 Adding or changing an asset
1. Take it from its original source under an allowed licence; record the exact version.
2. Confirm it is not from Adobe (§1.1). Keep the licence text.
3. Add the file, its licence file and an `ATTRIBUTION.toml` entry. Run
   `cargo xtask assets --write`, then `cargo xtask assets`.
4. If the origin or licence is unclear, **don't add it**; ask the maintainer.

If you find an asset that breaks these rules, remove it from the repository and every output, then
tell the maintainer.

---

## 2. Clean-room rules
- Never read, disassemble or copy anything inside an Adobe product.
- Behaviour comes only from public specifications (ISO 32000-2, the Arlington PDF model) and public
  documentation.
- Never copy GPL/AGPL code. Copyleft tools may run only as separate external processes for testing.
- Never commit personal data, confidential MGRC documents or large real-world test files. Test
  documents are small and synthetic.

## 3. Automation and agent control
- **Everything is reachable without the GUI.** Each user-facing feature has an engine API and a tool
  in `crates/automation` (shared by `selfpdf-cli run` and MCP), with an end-to-end test.
- **The UI control channel is opt-in.** It runs only when the app starts with `--control <file>`,
  listens on loopback only, and uses a random per-launch token written to an owner-only file. Never
  enable it by default, widen the bind address, or log the token.
- **The MCP server is opt-in.** Nothing starts or registers it automatically, and it opens no network
  port. It runs only while a user or their agent launches `selfpdf-cli mcp`.
- **Safe by default.** Tools never discard unsaved work silently, write atomically, honour `--root`,
  and return errors that say what was wrong and what is allowed.

## 4. Never crash
MGRC-PDF opens files from outside MGRC, and staff trust it with their work. A malformed PDF, a bad
argument, a corrupt settings file or a full disk must produce an actionable error, never a crash and
never lost work. **This outranks feature work.**

- **Fail with `Result`** through the crate's error type and `?`. Lenient readers fall back and record
  the repair.
- **No panicking shortcuts** outside tests (`unwrap`, `expect`, `panic!`, `unreachable!`, `todo!`,
  `unimplemented!`) unless provably infallible, with a comment saying why.
- **No `unsafe`.** The workspace sets `unsafe_code = "forbid"`.
- **Treat input-derived numbers as hostile:** `get()` instead of `[i]`, slice strings at char
  boundaries, checked/saturating arithmetic, guard division by zero and NaN/inf, cap allocations.
- **Bound recursion** with seen-sets or depth limits; object graphs can be cyclic.
- **Don't cascade:** handle lock poisoning, and keep the last-resort `catch_unwind` guards around
  untrusted-document code (keep `panic = "unwind"`).
- **Prove it:** every crash fix ships with a small synthetic regression test that failed before the
  fix. Turn fuzzer findings (`cargo xtask fuzz`) into such tests first.
- **Enforced by clippy:** `clippy.toml` lets tests use `unwrap`/`expect`/`panic!`; clean crates deny
  them with `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, …)]`.

## 5. Everything else
Follow `CLAUDE.md`: quality checks, branches, and how to run and look at the app.
