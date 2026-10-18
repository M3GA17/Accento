ACCENTO — MACOS BETA 0.5

Drag Accento.app to Applications before granting permissions. Use a build for
your processor, or the universal build for Intel and Apple Silicon. The beta uses
an ad-hoc signature and has not been notarized; Apple has not verified the publisher.
If macOS blocks this downloaded beta, use Privacy & Security > Open Anyway after
attempting to open the trusted app. Do not disable Gatekeeper globally.

Enable Accento under System Settings > Privacy & Security > Accessibility.
macOS may also require Input Monitoring permission. Then use Turn Accento on
in settings; if macOS requires a restart, quit and reopen the installed app.
Permission grants belong to the installed application identity and can be affected
by moving or replacing an unsigned app.

Hold a letter, tap Space, wait for the glass keys and release the letter to insert
the selected character. Left/Right or another Space navigates; Esc cancels.
Accento uses a Quartz event tap and tagged Unicode events, without replacing
the clipboard. Secure Event Input and accessible secure text fields disable a gesture.

Preferences: ~/Library/Application Support/Accento/settings.json.
Application exclusions use executable process names such as Safari or TextEdit,
rather than a .app path. Exit is available in settings and in the menu-bar icon.

Read TEST-MACOS.it.txt for setup, the test checklist and the feedback format.
Use Accento.app/Contents/MacOS/Accento --diagnostics for version, architecture and
permission status without starting typing hooks or reading document contents.
Read PLATFORMS.md for the verified scope and limits. Localized quick-start guides
were originally written for Windows; follow this file for macOS setup. Original
dependency license texts are in licenses/. Accento does not log typed text or
use network services. Native build/startup checks do not verify live typing or
the permission flow; those tests are the purpose of this beta.
