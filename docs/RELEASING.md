# Preparing an Accento release

**Current status: local preparation only. No repository, tag or GitHub release is created by the packaging scripts. Publication is on hold.**

## Scope

`0.5.0-beta.1` is the proposed first public beta number. Historical reports use the earlier internal development numbering. The source now includes Windows, Linux X11 and macOS backends. Windows and Linux have local execution checks; the macOS backend is type-checked for Intel and Apple Silicon, while a complete app build and permission/live-input checks still need a Mac. Native Wayland input is not implemented. Read [the platform verification scope](cross-platform.md) before deciding which binaries may be published.

The intended GitHub destination is `M3GA17/Accento`, subject to the later publication decision. The local repository currently has no configured remote.

## Prepare locally

1. Align `Cargo.toml`, the three workspace package versions in `Cargo.lock`, `package.json`, its root lockfile entries, `crates/accento-windows/tauri.conf.json`, and the numeric Windows manifest version. The UI reads its version from `package.json`.
2. Update the usage guide, proposed release notes and README status. Keep historical verification reports intact.
3. Run `npm run bundle`. See [DEVELOPMENT.md](DEVELOPMENT.md) for prerequisites and checks.
4. Inspect the installer, portable archive and `SHA256SUMS.txt`. Both distributions must include the usage guide, 47 quick-start guides, coverage sources, Accento license, dependency notices and original license texts.
5. Verify the ZIP's executable byte for byte against `target/release/Accento.exe`. Inspect installer payload and documentation. Test the extracted portable with its own temporary preferences and WebView2 profile.
6. Refresh README captures using `node scripts/capture-docs.mjs` and inspect the images. Check all relative Markdown links.

The installer is unsigned and installs for the current user. It can download WebView2 if absent. The portable requires an existing runtime. Neither distribution has automatic updates. Preferences are stored separately in `%LOCALAPPDATA%\Accento`.

## Later publication

Publication needs a new explicit instruction; the following is a checklist, not part of the build script.

1. Confirm the final version, platform scope, repository name and visibility. Check existing tags/releases and never overwrite an already-published version.
2. Review the source changes to include, preserve unrelated local work and check the intended GitHub repository for build workflows before pushing. No release workflow is added by this preparation.
3. Commit the reviewed source, push the chosen branch and create an annotated `v0.5.0-beta.1` tag at that exact commit.
4. Create a draft GitHub release with the verified tag and notes, marked **prerelease**, and upload the installer, portable ZIP and checksum file. Do not mark a beta as the latest stable release.
5. Download the draft assets to a fresh directory and verify hashes, sizes, names and the tag commit.
6. Publish only after the scope and draft assets have been checked. Update the README status and download links at the same time.

The generated source archives are supplied by GitHub from the tag. Build files, staged dependencies and portable binaries stay outside Git source control.
