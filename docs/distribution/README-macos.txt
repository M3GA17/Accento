ACCENTO — MACOS DEVELOPMENT BUILD

Drag Accento.app to Applications before granting permissions. Use a build for
your processor, or a universal build if provided. These local builds are unsigned
and have not been notarized. A signed/notarized distribution is a separate release
step, and no public release is currently available.

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

Read PLATFORMS.md for the verified scope and limits. Localized quick-start guides
were originally written for Windows; follow this file for macOS setup. Original
dependency license texts are in licenses/. Accento does not log typed text or
use network services. Full macOS app and input verification requires a Mac.
