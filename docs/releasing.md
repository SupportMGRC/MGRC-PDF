# Releasing MGRC-PDF

How a new version of MGRC-PDF gets from the `Noor` branch to staff PCs.

## Branches

| Branch | Purpose |
|---|---|
| `Noor` | Day-to-day work. Every change is committed and pushed here first. |
| `main` | Release branch and GitHub default branch. Updated only by merging `Noor`. |

```sh
git checkout Noor
# ...change, then run the quality checks in CLAUDE.md...
git add -A && git commit -m "describe the change" && git push

git checkout main && git merge Noor && git push
```

## 1. Set the version

The version lives in one place, `[workspace.package] version` in the root `Cargo.toml`. It is shown
in **About** and stamped into the installer.

```sh
cargo xtask version              # print the current version
cargo xtask version set 0.8.1    # updates Cargo.toml and Cargo.lock
```

Commit the change on `Noor`, then merge it into `main`.

## 2. Build the Windows installer

The workflow `.github/workflows/release.yml` (**Release (MGRC Windows)**) builds on a GitHub Windows
machine and produces:

| File | What it is |
|---|---|
| `selfpdf-<version>-windows-x64.msi` | Installer for all users (Program Files, Start menu, desktop shortcut, "Open with") |
| `selfpdf-<version>-windows-x64-portable.zip` | No-install edition: `selfpdf.exe`, `selfpdf-cli.exe`, OCR models |

**Test build:** GitHub → **Actions → Release (MGRC Windows) → Run workflow**, branch `main`. When it
finishes (about 30–60 minutes), download **selfpdf-windows-x64** from the run's *Artifacts*.

**Official release:** tag the commit on `main`. The same build runs and attaches both files to a
GitHub Release named `MGRC-PDF v<version>`.

```sh
git checkout main
git tag v0.8.1
git push origin v0.8.1
```

GitHub Actions minutes on a private repository are limited per month, and Windows builds count
double, so use test builds sparingly.

### Building on a Windows PC instead

1. Install Rust (rustup, MSVC toolchain), Visual Studio Build Tools with *Desktop development with
   C++* (provides the Windows SDK), and WiX:
   `dotnet tool install --global wix --version 5.0.2`.
2. From the repository folder in PowerShell:

   ```powershell
   pwsh packaging/windows/package.ps1 -Arch x64
   ```

3. The MSI and portable zip are written to `dist\release\`.

`package.ps1` builds both programs with a static C runtime (no Visual C++ redistributable needed),
checks their headers, fetches the OCR models, builds and inspects the MSI (`test-msi.ps1`), signs if a
certificate is configured, and builds the portable zip. `-Arch x86` and `-Arch arm64` also work.

## 3. Code signing (optional)

Unsigned installers work, but Windows SmartScreen warns when a user runs one by hand. Deployment
through Intune or Group Policy shows no warning.

To sign, obtain a code-signing certificate for Malaysian Genomics Resource Centre Berhad and add two
repository secrets (GitHub → **Settings → Secrets and variables → Actions**):

| Secret | Value |
|---|---|
| `WINDOWS_CERTIFICATE` | the `.pfx` file, base64-encoded |
| `WINDOWS_CERTIFICATE_PASSWORD` | its password |

`packaging/windows/sign.ps1` then signs both programs and the MSI automatically.

## 4. Deploy to staff

- **Intune / Group Policy (recommended):**

  ```bat
  msiexec /i selfpdf-<version>-windows-x64.msi /qn ALLUSERS=1
  ```

  A newer MSI upgrades the installed version in place; settings are kept.
- **By hand:** share the MSI; users double-click it (admin rights required).
- **No admin rights:** share the portable zip.

## 5. Check before announcing

- Install the MSI on a test PC: the Start menu entry opens MGRC-PDF, **About** shows the new version,
  the MGRC logo and "Developed and maintained by Noor".
- Open, edit and save a sample PDF; switch between light and dark mode.
- Confirm **Edit with MGRC-PDF** appears when right-clicking a PDF.

## Installer identity

The MSI is published by *Malaysian Genomics Resource Centre Berhad* and has its own `UpgradeCode`
in `packaging/windows/pdfcraft.wxs`. **Never change the UpgradeCode**: Windows uses it to recognise
newer versions of MGRC-PDF and upgrade them in place.

## Other platforms

Packaging scripts for macOS (`packaging/macos/package.sh`), Linux (`packaging/linux/package.sh`:
AppImage, `.deb`, `.rpm`, tarball) and FreeBSD (`packaging/freebsd/package.sh`) are kept in the
repository and can be run by hand on those systems. They are not part of MGRC's release workflow.
