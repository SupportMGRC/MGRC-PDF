# Changelog

All notable changes to MGRC-PDF are listed here. Versions follow MAJOR.MINOR.PATCH.

## [Unreleased]

### Fixed
- Printing no longer closes MGRC-PDF when the printer driver crashes (seen on the RICOH IM C2010
  with the Microsoft IPP Class Driver). Print jobs on Windows now run in a separate helper process;
  if the driver fails, MGRC-PDF stays open, shows what went wrong, and records it in pdfcraft.log.

## [0.2.0] - 2026-10-19

### Fixed
- Print preview now shows pages in colour; it was always grey even when Print in grayscale was off.
- Help ▸ Check for updates now checks MGRC-PDF's own releases.
- PDFs from some suppliers (for example Bannershop invoices) showed empty boxes instead of
  text. Their embedded fonts carry an invalid setting written by the software that created them;
  MGRC-PDF now corrects it automatically when drawing the page. The PDF file itself is not changed.
- Saving a file keeps its original creation date, and keeps the permissions of the file it replaces.
- Saving refuses to replace a read-only file instead of failing part-way.
- Save As to a new file writes a clean copy without earlier revisions.
- Removing security also works on files whose encryption settings are stored in an unusual place.
- Printing keeps layers that are hidden in the Layers panel hidden.
- Closing the app while it is minimised no longer saves a zero-size window for the next start.
- Scanned text (OCR) no longer changes how the rest of the page's text is drawn.
- Many smaller fixes when reading damaged or unusual PDFs (fax images, cross-reference tables,
  fonts without width tables, forms with dynamic layouts).

### Improved
- Faster scrolling with a mouse wheel, smoother page turns in single-page view, and faster
  drawing of scanned images when zooming.
- Type an exact zoom level; the Pages panel follows the current page; Go to page brings the page
  fully into view.
- Ctrl+drag copies pages in the Organize grid; reverse the page order of a document.
- Editing text: better line wrapping, the cursor matches the line height, the chosen font is
  applied to the paragraph being edited, and edits near watermarks keep paragraphs separate.
- Forms: comb boxes are detected as one field, button labels can be edited, form data export keeps
  whitespace and full values.
- Redaction finds more personal data (card numbers with expiry/CVV, unusual e-mail addresses) and
  also removes replies and hidden structure linked to removed content.
- Creating a PDF from photos respects the camera's orientation; plain text keeps its indentation.
- Word, HTML and PowerPoint export no longer turn running text into table rows; exported images
  record their resolution.
- Attachment comments can be saved or opened from their menu.
- PDF thumbnails in File Explorer no longer show the app icon over the corner of the page.
- Translations updated for several languages.

### Changed
- About and Help show only MGRC-PDF's own links.
- Removed build checks for platforms MGRC does not release (FreeBSD, Windows on ARM, Nix) and
  automatic dependency-update requests.
- The version tool (`cargo xtask version set`) now updates every internal version entry in one step.

### Upgrading
- Install `selfpdf-0.2.0-windows-x64.msi` over v0.1.0; it upgrades in place and keeps settings.

## [0.1.0] - 2026-10-16

First release of MGRC-PDF for Malaysian Genomics Resource Centre Berhad.

### Added
- MGRC branding: logo on the home screen and in About, MGRC app icon, and the
  "Developed and maintained by Noor" credit line.
- Light and dark mode (follows Windows by default).
- Windows installer (`selfpdf-<version>-windows-x64.msi`) published by
  Malaysian Genomics Resource Centre Berhad, plus a portable edition that needs no installation.
- Automated Windows build and release through GitHub Actions.
- Documentation written for MGRC.

### Changed
- Program named `selfpdf.exe`, command-line tool `selfpdf-cli.exe`, app ID `selfpdf`.

### Removed
- Discord and other external community links and branding.
