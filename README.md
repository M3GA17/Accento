# Accento

Accento è un'utility Windows autonoma per inserire accenti e varianti mentre scrivi. Il motore è scritto in Rust; il selettore mostra solo tasti Liquid Glass indipendenti su una finestra trasparente, con alternativa solida.

La versione 0.3.0 è un prototipo portabile per Windows 11 x64, con la UI del nuovo FigCraft implementata: popup senza pannello o label, quattro sezioni nelle impostazioni, temi chiaro/scuro e layout compatto. Non richiede PowerToys, .NET o WebView. Il sistema iniziale previsto è Windows 11 build 22621 o successiva; l'ambiente verificato è la build 26300.

## Stato del progetto

Rework del 15 ottobre 2026: candidati 64 × 64, gap trasparenti, materiale con sfocatura e rifrazione del backdrop campionato all'apertura; impostazioni Scrittura, Lingue, Aspetto e Applicazioni con controlli nativi e footer stabile. Il motore del gesto è invariato. Sono superati 21 test Rust e 35 controlli Windows, inclusa la verifica dell'alpha effettivo del popup: [rapporto 0.3.0](docs/verifiche-0.3.0.md).

Il [FigCraft del design 0.3](design/figcraft/Accento-Design-System.figcraft) conserva le schermate editabili e il prototipo. Le catture del prodotto implementato sono in [evidenze/0.3.0](docs/evidenze/0.3.0). [Contenuti, verifica e rigenerazione](design/figcraft/README.md).

## Avvio e utilizzo

Apri [Accento 0.3.0](dist/Accento-0.3.0-windows-x64/Accento.exe), oppure estrai [il pacchetto portabile](dist/Accento-0.3.0-windows-x64.zip) generato dalla compilazione. Chiudi prima un'eventuale istanza precedente dal menu dell'icona. Si apre la finestra impostazioni con un campo di prova. La chiusura della finestra lascia l'app nell'area di notifica; dal menu dell'icona scegli Esci per terminarla.

Tieni `e` premuta, premi e rilascia Spazio, attendi l'apertura e premi Destra. Rilascia `e` per inserire `è`. Esc annulla. Per le maiuscole inizia con Shift o Caps Lock. [Guida completa e limiti](docs/uso.md).

## Compilazione

Occorrono Rust con toolchain MSVC x64, gli strumenti C++ di Visual Studio e Windows SDK con `rc.exe`. Le dipendenze sono fissate in `Cargo.lock`.

```powershell
cargo run -p accento-windows
cargo test --workspace --locked
./scripts/build.ps1
```

`./scripts/build.ps1 -Test` include il collaudo interattivo Windows prima di preparare il pacchetto. Il collaudo usa una propria finestra: lascia libera la tastiera finché termina. Il normale avvio non esegue questi test e non salva catture o cronologia.

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
- `crates/accento-windows`: integrazione Win32, rendering, impostazioni, tray e collaudo.
- `docs`: prodotto, analisi REA, piano, guida ed evidenze.

Licenza MIT; dipendenze e avvisi nel file [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md).

