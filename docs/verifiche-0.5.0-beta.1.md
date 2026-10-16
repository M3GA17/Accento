# Preparazione locale di Accento 0.5.0-beta.1

Data: 16 ottobre 2026. Sistema di verifica: Windows 11 x64. **Nessuna pubblicazione GitHub, nessun tag e nessun push eseguiti.**

## Risultati

- Build React/TypeScript e release Rust/Tauri con risorse incorporate: superate.
- 8 scenari UI Playwright, inclusi tema scuro, selettore lingue, set, validazione, bozza, BlackList, layout compatto e accessibilità: superati.
- 39 test Rust: 22 del motore/catalogo e 17 di integrazione locale: superati.
- Formattazione Rust e Clippy con `-D warnings`: superati.
- 9 verifiche Tauri/WebView2 reali e 47 coppie lingua/set: superate sull'eseguibile di build, sulla copia portabile e sul payload estratto dall'installer. Hook tastiera disabilitati, preferenze e profilo WebView2 temporanei.
- Collaudo delle vecchie impostazioni native `--ui-test`: superato, inclusa l'assenza di ridisegni superflui al limite dello scorrimento.
- Installer NSIS x64 generato per l'utente corrente, con scelta italiano/inglese e gestione del prerequisito WebView2. Archivio NSIS verificato ed estratto con 7-Zip senza errori.
- ZIP portabile: 502 file, di cui 47 guide localizzate. Tutti i contenuti decompressi confrontati tramite SHA-256 con la cartella portabile; eseguibile identico alla build verificata.
- Documentazione e licenze presenti sia nel portabile sia nell'installer e confrontate byte per byte. L'eseguibile nell'installer differisce soltanto nei tre byte del marcatore Tauri del tipo di bundle (`NSS` invece di `UNK`), come previsto dalla procedura NSIS.
- Checksum dei due pacchetti ricalcolati e confrontati con `SHA256SUMS.txt`: corrispondenti.
- README inglese e italiano con copertina, logo e schermate: immagini ispezionate e collegamenti relativi ai file verificati.

## File preparati

Cartella: `dist/releases/v0.5.0-beta.1/`.

| File | Dimensione | SHA-256 |
| --- | ---: | --- |
| `Accento_0.5.0-beta.1_windows-x64-setup.exe` | 1.912.174 byte | `4638e4057e37bb74892b4620c7e2b2ca8e6a8d13cc37dfd7fce57c741f97f7a1` |
| `Accento_0.5.0-beta.1_windows-x64_portable.zip` | 3.094.171 byte | `3fa69d5caa95ef14751f8ddd1a4c1785db9547c236e5555426bcf3afb60b4067` |

I file generati restano fuori dal controllo versione. Le risorse condivise dell'installer e del portabile vengono preparate da `scripts/package-windows.ps1`; lo script verifica anche la versione del prodotto e l'integrità del portabile prima di produrre i checksum.

## Limiti della verifica

Non è stata eseguita un'installazione/disinstallazione interattiva nel profilo personale: il controllo dell'installer riguarda archivio, payload, licenze e avvio dell'eseguibile estratto. Il download/installazione di WebView2 su una macchina che ne è priva resta da provare in un ambiente pulito.

Il collaudo con digitazione sintetica `--self-test` non è stato rieseguito per questa preparazione. Restano i limiti già documentati per applicazioni destinatarie, layout, IME, campi protetti, privilegi e Narrator. La beta non è firmata e non include aggiornamenti automatici. Linux e macOS richiedono ancora le implementazioni native: non sono stati prodotti pacchetti per questi sistemi.

La numerazione pubblica proposta parte da 0.5.0-beta.1; i rapporti interni precedenti contrassegnati 0.6.0 restano storici.
