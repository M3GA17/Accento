ACCENTO — WINDOWS BETA
Accents and character variants, right where you type.

REQUIREMENTS
Windows 11, 64-bit Intel/AMD. Microsoft Edge WebView2 Runtime.
No PowerToys, .NET, Node.js or Rust installation is needed to run Accento.

INSTALLER
Run the setup executable. It installs for the current user and can download
Microsoft WebView2 if it is missing (internet access may be needed at setup).
Uninstall from Windows Settings > Apps. User preferences are kept separately.

PORTABLE
Extract the entire ZIP to a writable folder, then open Accento.exe.
WebView2 must already be installed. Keep the guides and license files beside
the executable. Preferences are saved in %LOCALAPPDATA%\Accento\settings.json,
not in the portable folder. To remove Accento, exit it from the notification
area and delete the extracted folder.

FIRST ACCENT
1. Hold e.
2. Press and release Space while still holding e.
3. Wait for the glass keys. The first character is already selected.
4. Use Left/Right or press Space again to select another character.
5. Release e to insert it. Esc cancels.

SETTINGS
Choose character sets and interface language independently. Set the opening
delay, exclude applications by executable name, or pause Accento from its icon.
Closing settings keeps Accento running. Choose Exit from the notification icon
to stop it. Pause other tools using the same shortcut while trying Accento.

BETA STATUS
The beta is unsigned. Windows may display an unknown-publisher warning.
Only run binaries obtained from a source you trust; SHA256SUMS.txt lets you
compare downloads with the supplied checksums. No automatic updater is included.
Linux and macOS native integrations are not included in this Windows build.
Translations are drafts awaiting native-speaker review. Individual apps,
keyboard layouts, password fields, IMEs and elevated apps have limitations;
read LEGGIMI.md before relying on the utility in a particular workflow.

PRIVACY
Accento works offline after its runtime is installed. It does not record typed
sentences, key history or field contents. Glass backdrop samples stay in memory.

DOCUMENTATION
LEGGIMI.md                     Complete Italian usage guide and limitations
copertura-europa.md             Language coverage and links to 47 local guides
guide/                         Localized quick-start guides (draft translations)
LICENSE                        Accento source license
THIRD-PARTY-NOTICES.md          Third-party notices
DEPENDENCIES-UI.md              UI dependency inventory
LICENSE-UNICODE.txt, licenses/  Original dependency license texts

PROJECT
https://github.com/M3GA17/Accento
