ACCENTO — LINUX X11 DEVELOPMENT BUILD

Install the .deb on a compatible Debian/Ubuntu desktop, or make the AppImage
executable and open it. The current typing backend requires a real X11 session.
XWayland does not provide access to native Wayland applications; in a Wayland
session Accento leaves typing disabled and displays an explanation in settings.

Hold a letter, press and release Space, wait for the glass picker, then release
the letter to insert the selected character. Left/Right or another Space selects
another variant. Esc cancels. Other accent utilities can conflict with this gesture.

The XInput2 backend captures physical X11 keyboards and forwards ordinary input
through XTEST. No root process, input-group membership, evdev or uinput access is
required. Preferences are in $XDG_CONFIG_HOME/accento/settings.json, or
~/.config/accento/settings.json if XDG_CONFIG_HOME is not set.

Use the executable process name for exclusions, for example firefox or code.
Use Exit in settings or the notification menu to stop Accento. Some desktops
require an AppIndicator extension to display the notification icon.

Read PLATFORMS.md for the verified scope and remaining limitations. The localized
quick-start guides were originally written for Windows; platform setup is here.
The app does not log typed text or use a network service. Runtime dependencies
and their licenses are listed in THIRD-PARTY-NOTICES.md and licenses/.
