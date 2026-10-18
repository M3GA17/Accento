# Releasing Accento

The release being prepared is `0.5.0-beta.3`, a Windows x64 and universal macOS prerelease in the private `M3GA17/Accento` repository. Packaging scripts prepare files; GitHub publication is manual. The manually dispatched `Build macOS beta` workflow produces and checks Mac packages without publishing a release.

The source includes Windows, Linux X11 and macOS backends. Read [the platform verification scope](cross-platform.md) before deciding which binaries to publish. Beta 1 notes and verification reports describe historical local preparation.

## Beta 3 before stable 0.5.0

Beta 3 distributes Windows and a universal Mac app before live Mac testing. Build/package checks and native WebView/IPC startup checks run unattended on Apple Silicon and Intel Mac runners with keyboard hooks disabled. Ad-hoc signing permits this test distribution without a Developer ID certificate; notarization and live input are not certified by these checks.

Send the Mac package and `docs/distribution/TEST-MACOS.it.txt` to the tester. Collect the machine/OS/layout, permission state, application and exact gesture for each result. Fix reported issues and recheck affected packages before preparing stable `0.5.0`; publishing the beta does not automatically promote it to stable.

## 0.5.0 stable release scope

The stable `0.5.0` release targets **Windows and macOS**, as agreed on 18 October 2026. The published `0.5.0-beta.2` remains a Windows-only prerelease.

| Target | Planned distributions | Required verification |
| --- | --- | --- |
| Windows 11 x64 | Current-user installer and portable ZIP | Final-build typing tests in representative applications, clean-machine installation with and without WebView2, upgrade from beta 2 with preferences preserved, removal, accessibility and multiple-monitor DPI checks |
| macOS 13 or newer, Intel and Apple Silicon | DMG and zipped app, universal or separate architecture builds | Complete native app builds and live input on both architectures; Accessibility/Input Monitoring permission grant, denial and recovery; signing/notarization and Gatekeeper checks; installation, upgrade, removal, VoiceOver and multiple-monitor checks |

Both platforms must pass shared Rust and UI checks and preserve the same gesture, character catalog, exclusions and local preferences. Record native verification against the final release commit and verify each distributable's contents and SHA-256 checksum. The configured minimum OS versions still require dedicated validation.

Linux X11 and Wayland are deferred to a later release. Existing Linux source and developer checks remain available; Linux packages and a Wayland backend are outside the `0.5.0` release requirements.

## Prepare and verify

1. Align `Cargo.toml`, the three workspace package versions in `Cargo.lock`, `package.json`, its root lockfile entries, `crates/accento-windows/tauri.conf.json`, and the numeric Windows manifest version. The UI reads its version from `package.json`.
2. Update the usage guide, release notes and README links. Keep historical verification reports intact.
3. Run `npm run bundle` on each target platform. On Windows this builds the frontend, runs browser and Rust checks, builds the NSIS installer, checks the desktop app and creates the portable ZIP and SHA-256 manifest. On macOS it runs the shared checks and builds the native bundles; use the target options in [cross-platform.md](cross-platform.md#macos-setup) for a universal app. Native permission and typing verification are separate from packaging. See [DEVELOPMENT.md](DEVELOPMENT.md).
4. Inspect every platform package and its `SHA256SUMS.txt`. All distributions must include usage documentation, 47 quick-start guides, coverage sources, Accento's license, dependency notices and license texts.
5. Verify extracted and installed applications on their native systems. For Windows, use isolated preferences and WebView2 data when checking the portable. For macOS, complete the permission and live-input checks with the installed app. Check package contents and archive integrity on both platforms.
6. Refresh Windows README captures with `node scripts/capture-docs.mjs`, inspect them and check relative Markdown links. For Mac, dispatch `.github/workflows/build-macos.yml` at the reviewed commit, collect the universal packages and both native verification reports, and combine the platform package checksums into the release manifest. Add verified macOS setup instructions and native captures for the stable release.

The current Windows beta installer is unsigned and installs for the current user. It can download WebView2 if absent. The Windows portable requires an existing runtime. Neither beta distribution has automatic updates. Windows preferences are stored separately in `%LOCALAPPDATA%/Accento`.

## Publish

Publication requires an explicit user instruction. For an authorized release:

1. Check the destination, visibility, existing tags/releases and remote workflows. Never overwrite a published version.
2. Commit the reviewed source, push the chosen branch and create an annotated version tag at that exact commit.
3. Create a draft GitHub release with the release notes, verified platform packages and checksum files. Mark beta versions as **prerelease** and never as the latest stable release. For stable `0.5.0`, include both Windows and macOS packages, complete the verification above and leave **prerelease** disabled.
4. Download the draft assets into a fresh directory. Compare hashes, sizes, names and the tag commit, then publish the verified draft.

GitHub supplies source archives from the tag. Build files, staged dependencies and portable binaries stay outside Git source control. Private release downloads require repository access.
