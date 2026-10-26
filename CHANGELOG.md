# Changelog

All notable changes to MGRC-PDF are listed here. Versions follow MAJOR.MINOR.PATCH.

## [0.3.0] - 2026-10-26

This release brings in a week of improvements from the open-source project MGRC-PDF is based
on (its changes from 19 to 25 October 2026), with MGRC branding kept throughout.

### Added
- **Update notice:** when MGRC-PDF starts, it checks once, quietly, for a newer MGRC-PDF release.
  If there is one, a bar under the mode bar says so, with **Download** (opens the release page)
  and **Later** (hides the bar until the next start). Installing needs an administrator on MGRC's
  PCs, so the bar says to ask IT if Windows asks for a password. Nothing shows when MGRC-PDF is
  up to date or offline, and nothing is downloaded or installed automatically.
- **Pages:** Extract pages takes typed page ranges; **Split document…** on
  the page thumbnails' right-click menu; a **Page tools** menu in the Pages panel; an optional
  gapless continuous page display; Full screen shows one fitted page at a time.
- **Editing:** paste a picture from the clipboard with Ctrl+V; drag to size a new text box; text
  boxes and callouts can have a fill and border width; page images rotate by any angle; Delete
  removes the selected paragraph or objects.
- **Header & Footer:** Times and Courier fonts, an option to shrink the page content out of the
  way, and previews that step through the pages (also for watermarks).
- **Comments:** flatten comments from the page's right-click menu or the Comments panel; arrow
  keys move the selected comment; drag a line's ends or a polygon's corners; the reply count
  shows under the Comments header.
- **Forms:** **Prepare a form ▸ Calculation order** (check and fix the order fields calculate
  in) and **Text field table**; nudge a field with the arrow keys; a **Today** button in the date
  picker; field tooltips on hover; editable drop-down lists accept a typed value; Tab and Space
  reach check boxes and radio buttons.
- **Document Properties:** a **Custom** tab for custom properties and a **Space Audit** tab;
  title and author stored only in XMP metadata now show.
- **Attachments:** attach a file to a document and delete one; open an attachment by
  double-click.
- **Bookmarks:** title colour and style; a new bookmark goes to the page you are on.
- **Layers:** Reset to Initial Visibility.
- **Measure:** type or pick an architect's drawing scale (1:50, 1:100 …).
- **All tools** opens as a full, searchable view that also finds menu commands.
- **Printing:** remembers the printer and paper size; printer driver options in the interface
  language; the US #10 envelope paper size; two-sided printing starts at the printer's own default;
  Print is unavailable on documents whose security forbids printing.
- **PDF Optimizer:** choose the target PDF version and object streams; remove PDF/A or PDF/X
  identification.
- **Compare** can filter text changes and visual differences.
- **Preferences:** always use my default zoom and page layout; restore the last view when
  reopening a document; field highlights on by default, with a toggle in Appearance.
- **12 new interface languages:** Albanian, Catalan, Central Kurdish, Finnish, Hindi,
  Indonesian, Korean, Lao, Norwegian Bokmål, Persian, Romanian and Thai.

### Improved
- Opening a file that is already open switches to its tab, and the running MGRC-PDF window comes
  to the front when another file is opened.
- Documents open at their own starting page and position, if they set one.
- Save As and export dialogs open in the document's own folder.
- Editing text keeps every space you type and keeps the original styling more reliably.
- Signatures: visible signatures read upright on rotated pages; certificate and time-stamp checks
  are stricter; only certificates that may sign documents are offered from the Windows store.
- Redaction also clears image pixels under the marks, follows nested content more thoroughly, and
  is applied on a copy, so a failure leaves the document as it was.
- Values typed into password fields are never written to any file.
- Word export keeps the page background colour and italic text in its paragraph.
- Large forms (XFA) with thousands of fields open in seconds; many XFA layout and date fixes.
- Many drawing fixes: fonts, colour spaces, transparency, shadings and patterns, check boxes and
  sticky notes without appearances, and Korean or Hebrew file names in tabs.
- Damaged or hostile PDFs are now held to strict limits on memory and time throughout, so they
  can't freeze or crash the app.

### Changed
- The interface library was updated to a newer version (egui 0.37).
- The mode bar no longer has a separate Read tab, matching Acrobat.
- Windows **Settings ▸ Apps** now shows MGRC-PDF's GitHub page as its support and update link
  (previously the original project's website).

### Upgrading
- Install `selfpdf-0.3.0-windows-x64.msi` over v0.2.x; it upgrades in place and keeps settings.

## [0.2.1] - 2026-10-19

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
