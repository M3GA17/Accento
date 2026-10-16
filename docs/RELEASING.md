# Releasing Accento

The current release is `0.5.0-beta.2`, a Windows x64 prerelease in the private `M3GA17/Accento` repository. Packaging scripts prepare local files; GitHub publication is manual. No release automation is enabled.

The source includes Windows, Linux X11 and macOS backends. Read [the platform verification scope](cross-platform.md) before deciding which binaries to publish. Beta 1 notes and verification reports describe historical local preparation.

## Prepare and verify

1. Align `Cargo.toml`, the three workspace package versions in `Cargo.lock`, `package.json`, its root lockfile entries, `crates/accento-windows/tauri.conf.json`, and the numeric Windows manifest version. The UI reads its version from `package.json`.
2. Update the usage guide, release notes and README links. Keep historical verification reports intact.
3. Run `npm run bundle`. This builds the frontend, runs browser and Rust checks, builds the NSIS installer, checks the desktop app and creates the portable ZIP and SHA-256 manifest. See [DEVELOPMENT.md](DEVELOPMENT.md).
4. Inspect the installer, portable archive and `SHA256SUMS.txt`. Both distributions must include usage documentation, 47 quick-start guides, coverage sources, Accento's license, dependency notices and license texts.
5. Verify the extracted portable with isolated preferences and WebView2 data. Check installer contents and archive integrity.
6. Refresh README captures with `node scripts/capture-docs.mjs`, inspect them and check relative Markdown links.

The installer is unsigned and installs for the current user. It can download WebView2 if absent. The portable requires an existing runtime. Neither distribution has automatic updates. Preferences are stored separately in `%LOCALAPPDATA%/Accento`.

## Publish

Publication requires an explicit user instruction. For an authorized release:

1. Check the destination, visibility, existing tags/releases and remote workflows. Never overwrite a published version.
2. Commit the reviewed source, push the chosen branch and create an annotated version tag at that exact commit.
3. Create a draft GitHub release marked **prerelease**, with the release notes, installer, portable ZIP and checksum file. Do not mark a beta as the latest stable release.
4. Download the draft assets into a fresh directory. Compare hashes, sizes, names and the tag commit, then publish the verified draft.

GitHub supplies source archives from the tag. Build files, staged dependencies and portable binaries stay outside Git source control. Private release downloads require repository access.
