# Verifica di Accento 0.5.0-beta.2

Data: 16 ottobre 2026. Ambiente: Windows 11 x64. Destinazione: repository GitHub privato `M3GA17/Accento`, prerelease `v0.5.0-beta.2`.

## Risultati

- `npm run bundle`: completato, con frontend TypeScript/Vite e build Rust/Tauri ottimizzata con risorse incorporate.
- 14 test UI Playwright: superati, inclusi funzionalità, accessibilità e 24 confronti di schermate a 1040×780 e 640×620. I 12 riferimenti a 1040 pixel sono aggiornati esclusivamente nel numero di versione (rettangolo x=151–156, y=750–756); quelli compatti sono invariati.
- 48 test Rust: 25 motore/catalogo, 13 piattaforma/preferenze e 10 Windows. Tutti superati.
- `cargo fmt --all -- --check` e Clippy con `-D warnings`: superati.
- 10 verifiche reali Tauri/WebView2 e 47 coppie lingua/set: superate sull'eseguibile compilato, sul portabile estratto dallo ZIP e sul payload estratto dall'installer. Incluse persistenza e attivazione indipendente delle valute, eventi pausa e protezione della bozza. Hook tastiera disabilitati, preferenze e profilo WebView2 temporanei.
- Collaudo delle impostazioni native `--ui-test`: superato.
- Installer NSIS: test integrità ed estrazione con 7-Zip superati. Il payload include tutte le risorse previste; il suo eseguibile differisce dalla build soltanto nei tre byte del marcatore Tauri `NSS` al posto di `UNK`.
- ZIP portabile: 504 file, di cui 47 guide localizzate. Tutte le voci estratte confrontate tramite SHA-256; eseguibile identico alla build.
- 503 risorse, documenti e licenze confrontati byte per byte con entrambe le distribuzioni.
- Checksum dei due pacchetti ricalcolati e confrontati con `SHA256SUMS.txt`: corrispondenti.
- Schermate README rigenerate dal vero eseguibile con profilo isolato; impostazioni e copertina ispezionate. Collegamenti relativi dei documenti di release controllati.

## Pacchetti

Cartella locale: `dist/releases/v0.5.0-beta.2/`.

| File | Dimensione | SHA-256 |
| --- | ---: | --- |
| `Accento_0.5.0-beta.2_windows-x64-setup.exe` | 1,959,833 byte | `6f65e06352ae8a5a649b388079463d12b44a5fcb9641d050e167b0130f2168c4` |
| `Accento_0.5.0-beta.2_windows-x64_portable.zip` | 3,170,460 byte | `9917b92e7fc73ea64e24904262e2b7247d1173451196a65cd8d7b5d0669656de` |

I binari restano fuori dal controllo versione. Gli allegati della bozza GitHub vengono scaricati nuovamente e confrontati con questi file prima della pubblicazione.

## Limiti

Non è stata eseguita un'installazione/disinstallazione interattiva nel profilo personale, né l'installazione di WebView2 su una macchina pulita. Il collaudo con digitazione sintetica `--self-test` non è stato eseguito in questa preparazione. Le verifiche isolate non certificano ogni applicazione, layout, IME, campo protetto o sessione remota.

La beta non è firmata e non include aggiornamenti automatici. La build segnala un bundle JavaScript superiore a 500 kB, senza errori di compilazione. Linux e macOS sono presenti nei sorgenti, con il precedente ambito di verifica documentato in [cross-platform.md](cross-platform.md); per questa release non sono stati compilati né allegati pacchetti per tali sistemi. Wayland nativo non è supportato.
