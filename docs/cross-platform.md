# Accento on Windows, Linux and macOS

Accento now has native input implementations for Windows, Linux X11 and macOS. They use the same deterministic gesture engine and catalog. The React settings and the preference schema are shared. **The 0.5.0-beta.2 prerelease ships Windows x64 binaries only, through the private [M3GA17/Accento repository](https://github.com/M3GA17/Accento/releases/tag/v0.5.0-beta.2).**

## Implementation and verification scope

| System | Input implementation | Verification in this workspace |
| --- | --- | --- |
| Windows 11 x64 | Existing Win32 hook, Unicode insertion and native glass picker | Rust tests, browser scenarios and native desktop integration |
| Linux X11 | XInput2 grabs of physical keyboards; normal events forwarded through the separate XTEST virtual keyboard; Unicode through Enigo's X11 backend | Native Linux build and real insertion into an isolated Xvfb/Tk field, including navigation, cancellation, quick Space, uppercase and auto-repeat |
| macOS Intel / Apple Silicon | Quartz event tap, Accessibility focus/protected-field checks, tagged UTF-16 events | Shared/native Rust module type-checked for both `x86_64-apple-darwin` and `aarch64-apple-darwin`; complete app build, permissions and live input still need a Mac |
| Linux Wayland | No global input backend yet | Typing stays disabled, with an explanation in settings; XWayland does not confer access to native Wayland apps |

Support here describes implemented source and the stated checks, not certification of every application, desktop or keyboard layout. The earlier local Windows beta packages predate this port and do not contain Linux/macOS executables.

## The gesture

Hold a letter, press and release Space, wait for the picker, then release the letter to insert the first selected variant. Left/Right or another Space navigates; Esc cancels. Starting with Shift or Caps Lock selects uppercase variants. Ending before the opening delay preserves ordinary typing. A gesture is cancelled when the observed target changes.

Windows retains its native picker. Linux and macOS use a transparent, non-focusable Tauri picker with independent glass cards, displayed near the bottom of the pointer's monitor. It does not take keyboard focus. Glass rendering and desktop compositing differ across systems; this is not a claim of identical optical refraction.

## Linux setup

Use an X11 session. No root process, `input` group membership, evdev or uinput access is needed: the backend uses the X server's normal client permissions. Another input utility may already own a keyboard; startup errors are shown in settings and typing remains paused. Keyboards are re-enumerated on XInput hierarchy changes. Ordinary events are forwarded through XTEST; the virtual forwarding device is excluded from capture to avoid feedback loops.

Building on Debian/Ubuntu needs a native Rust toolchain, Node/npm, `pkg-config`, GTK 3, WebKitGTK 4.1, AppIndicator, X11/XInput/XTest and xkbcommon development packages. For example:

```sh
sudo apt install build-essential pkg-config libgtk-3-dev libwebkit2gtk-4.1-dev \
  libayatana-appindicator3-dev librsvg2-dev libx11-dev libxi-dev libxtst-dev libxkbcommon-dev
npm ci
npx playwright install chromium
npm run bundle
```

The native configuration produces `.deb` and AppImage bundles. Build on the oldest Linux distribution you intend to support: a bundle built on a newer glibc is not automatically compatible with older distributions. This workspace's Linux execution tests use Ubuntu under WSL with a private Xvfb server. Some GNOME desktops need an AppIndicator extension for notification icons; **Exit** is also available in settings.

Wayland needs a separate compositor-supported input-method/capture integration. Accento does not request broad device permissions or claim native Wayland support through XWayland.

## macOS setup

The app configuration targets macOS 13 or newer. Install Xcode command-line tools, Rust and Node, then build on a Mac:

```sh
npm ci
npx playwright install chromium
npm run bundle
```

For a universal app, first install both Rust targets, then pass the target to the build command:

```sh
rustup target add aarch64-apple-darwin x86_64-apple-darwin
npm run bundle -- --target universal-apple-darwin
```

Move the app into Applications before granting **Privacy & Security → Accessibility** permission. macOS may also require **Input Monitoring**. If the event tap is denied, Accento starts its settings with typing paused and shows instructions. After granting permission, turn it on again; restart the installed app if required by macOS. Permissions belong to the application identity and may change when an unsigned build is replaced or moved.

The native module ignores its own tagged events, handles complete UTF-16 candidate strings without changing the clipboard and avoids Secure Event Input / accessible secure text fields. The complete app has not been built or run on a Mac in this workspace. Local signing/notarization and Gatekeeper validation are separate from the implemented backend and still need a native release environment.

## Preferences and exclusions

| System | Default preferences path | Exclusion example |
| --- | --- | --- |
| Windows | `%LOCALAPPDATA%\Accento\settings.json` | `notepad.exe` |
| Linux | `$XDG_CONFIG_HOME/accento/settings.json`, otherwise `~/.config/accento/settings.json` | `firefox` |
| macOS | `~/Library/Application Support/Accento/settings.json` | `Safari`, `Google Chrome` |

Exclusions use process executable names, never file paths or shell patterns. The UI and native validator apply platform-appropriate rules. `ACCENTO_CONFIG_DIR` explicitly overrides the directory for isolated checks; it should point to a private temporary folder during tests. Save operations retain the atomic replacement behavior.

The 47 localized quick-start guides were originally written for Windows; use this document for platform-specific setup. New platform hints fall back to English where a reviewed translation is not present.

## Limits that need native validation

- On Linux, focused X11 windows are observed; a focus change between internal widgets which does not change the X11 focus cannot always be detected. Protected fields are not generally detectable through X11. Pause Accento for sensitive or unsupported applications.
- macOS Accessibility metadata and Secure Event Input are checked, but coverage depends on what the target app exposes. Input-method compositions, dead keys and third-party keyboard layouts need dedicated native tests.
- On every platform, target applications can interpret or reject synthesized Unicode input differently. The catalog tests do not establish behavior in every editor, browser, terminal or remote session.
- Native macOS execution, clean-machine installers, signing and full desktop accessibility review remain distinct release gates. Do not publish all-platform binaries solely because the shared modules pass cross-compilation checks.

## Developer checks

```sh
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
npm run test:ui
```

The native typing test must run under a private X server, not a user's desktop:

```sh
env -u WAYLAND_DISPLAY XDG_SESSION_TYPE=x11 GDK_BACKEND=x11 \
  WEBKIT_DISABLE_DMABUF_RENDERER=1 dbus-run-session -- xvfb-run -a \
  python3 tests/native/linux-x11.py target/debug/Accento
```

It requires Tk, Xvfb and XTest, uses a temporary configuration, and drives only its controlled input fixture. Windows desktop tests keep their separate mutex/preferences/WebView2 profile. The module in `crates/accento-platform` can be checked for both Mac targets without claiming a complete Tauri app build.
