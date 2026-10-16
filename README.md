# Accento

Accento è un’utility Windows autonoma per inserire accenti e varianti mentre scrivi. Il motore è scritto in Rust; il popup mostra solo tasti Liquid Glass indipendenti su fondo trasparente.

La versione **0.6.0** usa **React 19 + TypeScript + Tauri 2** per le impostazioni. I componenti DimaLabs **Flat Product** includono pulsanti, campi e un selettore lingua ricercabile. Motore tastiera, icona nell'area di notifica e popup Liquid Glass restano nativi Windows in Rust.

La copertura comprende **47 voci linguistiche e varianti scritte nazionali europee**, incluse le scritture greca, cirillica, armena e georgiana. Ogni voce è presente nell’interfaccia, nei set e in una guida rapida. [Matrice di copertura, paesi, fonti e limiti](docs/copertura-europa.md). Le nuove traduzioni sono bozze complete da revisionare linguisticamente; non si dichiara una revisione madrelingua.

## Stato del progetto

Le impostazioni conservano Generale, Set caratteri e BlackList, tema **sempre scuro**, pausa immediata, bozza con Salva/Annulla e protezione dalla chiusura con modifiche non salvate. La chiusura distrugge la finestra web; motore e tray rimangono attivi. Le preferenze della versione precedente vengono riutilizzate.

Serve **Microsoft Edge WebView2 Runtime**; non servono PowerToys o .NET. L'interfaccia è incorporata nell'eseguibile e funziona offline. La migrazione non aggiunge compatibilità Linux o macOS al motore Windows. [Architettura, prove e limiti della migrazione](docs/migrazione-react-tauri.md).

Le schermate FigCraft precedenti restano un riferimento storico. Le catture correnti sono in [docs/evidenze/europe](docs/evidenze/europe).

## Avvio e utilizzo

Apri [Accento 0.6.0](dist/Accento-0.6.0-windows-x64/Accento.exe), oppure estrai [il pacchetto portabile](dist/Accento-0.6.0-windows-x64.zip) generato dalla compilazione. Chiudi prima un'eventuale istanza precedente dal menu dell'icona. Si apre la pagina Generale delle impostazioni. La chiusura della finestra lascia l'app nell'area di notifica; dal menu dell'icona scegli Esci per terminarla.

Tieni `e` premuta, premi e rilascia Spazio e attendi l'apertura: `è` è già selezionata. Rilascia `e` per inserirla. Destra o un nuovo Spazio avanzano alla variante successiva; Esc annulla. Per le maiuscole inizia con Shift o Caps Lock. [Guida completa e limiti](docs/uso.md).

## Compilazione

Occorrono Node.js/npm, WebView2 e Rust con toolchain MSVC x64, gli strumenti C++ di Visual Studio e Windows SDK con `rc.exe`. Le dipendenze sono fissate in `Cargo.lock` e `package-lock.json`; i pacchetti DimaLabs locali sono conservati in `vendor/`.

```powershell
npm ci
npm run desktop
cargo test --workspace --locked
./scripts/build.ps1
```

La compilazione verifica la UI React e il collegamento Tauri con preferenze temporanee e senza hook di input, oltre al vecchio collaudo nativo di regressione. `./scripts/build.ps1 -Test` include il collaudo interattivo Windows prima di preparare il pacchetto. Il collaudo usa una propria finestra: lascia libera la tastiera finché termina. Il normale avvio non esegue questi test e non salva catture o cronologia.

## Documentazione

| Documento | A cosa serve |
| --- | --- |
| [Prodotto e perimetro](docs/prodotto.md) | Obiettivi, destinatari e contenuto della prima versione |
| [Interazione e requisiti](docs/interazione.md) | Attivazione, selezione, annullamento e casi delicati |
| [Architettura proposta](docs/architettura.md) | Confini del motore Rust, integrazione Windows e opzioni grafiche |
| [Direzione visiva](docs/design.md) | Materiali, gerarchia, accessibilità e impostazioni |
| [Piano e verifiche](docs/piano.md) | Sequenza di sviluppo e condizioni per procedere |
| [Fonti e riferimenti](docs/fonti.md) | Quick Accent, API e ricerca DimaLabs |
| [Analisi Quick Accent](docs/reverse-engineering-quick-accent.md) | Evidenze dei binari installati, sorgente fissato e limiti dell'analisi |
| [Piano di ricreazione Rust](docs/piano-ricreazione-rust.md) | Contratto del gesto, struttura Rust, esperimento Glass e consegne |
| [Guida d'uso](docs/uso.md) | Avvio, configurazione e limiti del prototipo |
| [Verifiche della versione 0.1.0](docs/verifiche-0.1.0.md) | Ambiente, test eseguiti e lavoro ancora aperto |

## Struttura

- `crates/accento-core`: stati, timer e catalogo, senza dipendenze Windows.
- `crates/accento-windows`: Tauri, collegamento alle preferenze, integrazione Win32, rendering, tray e collaudo.
- `ui`: impostazioni React / Flat Product.
- `vendor`: pacchetti DimaLabs locali riproducibili.
- `localization`: registro nazionale, snapshot CLDR, traduzioni e provenienza.
- `docs/locales`: 47 guide rapide localizzate.
- `docs`: prodotto, analisi REA, piano, guida ed evidenze.

Licenza MIT; dipendenze e avvisi nel file [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md).

