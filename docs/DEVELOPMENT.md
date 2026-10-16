# Development

Accento has native implementations for **Windows 11 x64, Linux X11 and macOS**. The shared module is in `crates/accento-platform`; the existing desktop crate keeps its historical `accento-windows` directory name. Read [platform setup and verification scope](cross-platform.md) before building for Linux/macOS. Native Wayland typing is not implemented; the complete macOS app still requires verification on a Mac.

## Prerequisites

- Windows 11, Node.js 24 with npm, and Rust stable with the MSVC x64 toolchain.
- Visual Studio C++ Build Tools and Windows SDK, including `rc.exe`.
- Microsoft Edge WebView2 Runtime for the settings window.
- Playwright Chromium: `npx playwright install chromium` after `npm ci`.

The [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) describe the compiler and runtime setup. JavaScript and Rust dependencies are pinned in their lockfiles. The DimaLabs packages are vendored; a separate design-system checkout is not required.

## Run locally

```powershell
npm ci
npm run desktop
```

For the browser layout preview:

```powershell
npm run dev
# Open http://127.0.0.1:1421/?preview=1
```

Preview mode uses transient data. Without the explicit preview query, a regular browser cannot save desktop preferences or control the engine.

## Build the Windows packages

```powershell
npm run bundle
```

On Windows this performs the frontend build, nine browser scenarios, Rust formatting/tests/Clippy, a Tauri release build and NSIS packaging, an isolated native desktop integration test, and the historical native-settings regression test. It then assembles the portable folder and hashes the two distributable files. It **does not publish anything**. On Linux/macOS the same command runs native Rust/browser checks and Tauri packaging for the host, then collects the platform bundles and checksums; see [cross-platform instructions](cross-platform.md).

The [local beta verification report](verifiche-0.5.0-beta.1.md) records the current package checks, asset hashes and checks that still need a clean-machine installation test.

| Output | Path for the current local beta |
| --- | --- |
| Installer | `dist/releases/v0.5.0-beta.1/Accento_0.5.0-beta.1_windows-x64-setup.exe` |
| Portable ZIP | `dist/releases/v0.5.0-beta.1/Accento_0.5.0-beta.1_windows-x64_portable.zip` |
| SHA-256 manifest | `dist/releases/v0.5.0-beta.1/SHA256SUMS.txt` |
| Extracted portable | `dist/releases/v0.5.0-beta.1/Accento_0.5.0-beta.1_windows-x64_portable/` |

The installer uses Tauri's [NSIS packaging and WebView2 prerequisite handling](https://v2.tauri.app/distribute/windows-installer/). It installs for the current user, offers Italian and English, and ships the same documentation and dependency licenses as the portable ZIP. The app statically links the MSVC runtime. The portable ZIP requires an existing WebView2 runtime.

`npm run release:package` only reassembles assets from an already-built executable and installer. It checks that Cargo, npm, Tauri and the executable product version agree. To regenerate the resource folder before a direct Tauri build, run:

```powershell
./scripts/package-windows.ps1 -PrepareResources
npm run desktop:bundle
```

## Checks

```powershell
npm run build
./scripts/package-windows.ps1 -PrepareResources
npm run test:ui
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
node scripts/test-desktop.mjs target/release/Accento.exe
```

The desktop test uses a separate mutex, temporary preferences and an isolated WebView2 data folder. It disables keyboard hooks and exercises real Rust/WebView2 calls, persistence, all 47 locale/set pairs, pause events and unsaved-draft close protection. It does not type into the user's applications.

The optional `./scripts/build.ps1 -Test` also runs the synthetic typing self-test. That test opens a dedicated fixture window and sends keyboard input; keep the keyboard free while it runs. It is separate from the normal packaging checks and does not establish compatibility with every target application or physical keyboard layout.

## Update branding and screenshots

Edit `ui/public/brand/accento.svg`, then run `npm run brand:generate` to regenerate the Windows multi-resolution icon. The settings, browser icon, executable and notification icon share that mark.

After a release build:

```powershell
node scripts/capture-docs.mjs
```

This captures the real settings WebView2 with a temporary English example profile, then composes the README cover from those captures, the shared logo and a saved native picker fixture. The fixture comes from a controlled rendering test; it contains no personal desktop content. No clipboard or synthetic keyboard input is used.

## Source map

| Location | Responsibility |
| --- | --- |
| `crates/accento-core` | Character catalog and gesture state machine |
| `crates/accento-windows/src/input.rs` | Windows input translation and insertion |
| `crates/accento-windows/src/render.rs` | Native picker rendering |
| `crates/accento-windows/src/picker_material.rs` | Glass pixel material and backdrop treatment |
| `crates/accento-windows/src/desktop.rs` | Tauri lifecycle and typed preference commands |
| `crates/accento-platform/src/preferences.rs` | Platform paths, validation and atomic local persistence |
| `crates/accento-platform/src/linux.rs` | XInput2 device capture, forwarding and Unicode insertion |
| `crates/accento-platform/src/macos.rs` | Quartz/Accessibility input and focus integration |
| `crates/accento-windows/src/portable` | Shared Unix Tauri shell, tray and picker lifecycle |
| `ui/src` | React settings, translations and styles |
| `localization` | Language registry, pinned CLDR data and translation provenance |
| `scripts/build.ps1` | Local verification and build entry point |
| `scripts/package-windows.ps1` | Shared distribution resources, portable ZIP and checksums |

The public beta numbering starts at `0.5.0-beta.1`; earlier `0.6.0` files in historical verification reports describe internal development builds, not an existing public release.
