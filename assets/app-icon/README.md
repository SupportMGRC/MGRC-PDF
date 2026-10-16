# MGRC-PDF app icon

<img src="pdfcraft-small.svg" alt="MGRC-PDF app icon: the MGRC mark" width="128">

The app icon is the MGRC mark (white serif "MG / RC" on MGRC orange `#F7AF33`), centred on a
transparent 512 px tile. It is rendered from `docs/brand/mgrc-mark.svg`, a vector trace of the
company logo. Windows and Linux icons use the 512 tile; macOS icons put it on Apple's grid (an
824 px body centred on a transparent 1024 px canvas). The PDF document icon uses the same orange.

Regenerate every size with `packaging/icons.sh` (needs `resvg`). Licence: [LICENSE.txt](LICENSE.txt)
(MGRC trademark, not open source).

## Files

| File | What it is |
|---|---|
| `pdfcraft.svg` | the master vector (the MGRC mark on a 512 px tile); every PNG, `.ico` and `.icns` is rendered from it |
| `pdfcraft-small.svg` | a copy of the master used in this README |
| `pdfcraft-1024.png` | 1024 px on Apple's grid; also the runtime Dock icon on macOS |
| `pdfcraft.icns` | macOS icon (16–1024 px) |
| `pdfcraft.ico` | Windows icon (16–256 px), embedded in `selfpdf.exe` by `apps/pdfcraft/build.rs` |
| `pdfcraft-document.svg` | Windows PDF file icon master: a Lucide document outline, MGRC-orange badge and outlined PDF lettering; no font dependency |
| `pdfcraft-document.ico` | Windows PDF file icon (16–256 px), embedded as resource 2 in `selfpdf.exe` |
| `hicolor/<n>x<n>/apps/selfpdf.png` | Linux hicolor theme, 16–512 px; the 256 px one is the runtime icon on Windows and Linux |
| `hicolor/scalable/apps/selfpdf.svg` | Linux scalable icon (copy of the master) |

Where it shows: `apps/pdfcraft/src/main.rs` sets the window icon (Dock, taskbar, Alt-Tab, launcher) and the
Wayland app id `selfpdf`; `packaging/linux/selfpdf.desktop` names the
hicolor icon.

Windows PDF files use resource 2 (`"selfpdf.exe",-2`) when MGRC-PDF is chosen as the default
reader; the app keeps resource 1. The document outline is adapted from Lucide `file-text`
1.49.0 (ISC); its badge and outlined lettering are original (`MIT OR Apache-2.0`), covered by
[LICENSE-document.txt](LICENSE-document.txt). Both icons are embedded, requiring no runtime
sidecar. This provides a file type icon; first-page thumbnails still need a thumbnail provider.

## Regenerate

```sh
packaging/icons.sh        # needs resvg and python3; iconutil (macOS) for the .icns
cargo xtask assets        # then update the sha256 values in ATTRIBUTION.toml and run with --write
```
