# Impostazioni React / Tauri · Accento 0.6

> **Aggiornamento corrente:** Accento non usa più pacchetti DimaLabs. Button, TextBox e LanguageSelector sono sorgenti locali in `ui/src/components`; stili e tema scuro sono inclusi nel progetto. React Aria 1.22.1 è una dipendenza diretta. Gli archivi `vendor/` sono stati rimossi. Il confronto con 20 schermate precedenti verifica la parità visiva nei formati normale e compatto. Il seguito conserva il resoconto storico della migrazione.

Le impostazioni sono ora una UI React 19 / TypeScript visualizzata da Tauri 2 e WebView2. Button, TextBox e LanguageSelector provengono dai pacchetti DimaLabs reali; la presentazione web Flat Product è stata aggiunta alla libreria, insieme a un catalogo autonomo e ai temi chiaro/scuro derivati dall'export FigCraft Product / Flat.

## Confini e ciclo di vita

Tauri possiede il thread principale. Il precedente ciclo Win32 gira su un proprio thread STA e conserva hook della tastiera, worker, tray e popup Direct2D / DirectWrite. La finestra nativa delle impostazioni rimane nascosta e funge da destinatario dei messaggi; i suoi controlli non vengono costruiti nella modalità Tauri. `app/settings.rs` rimane per il collaudo nativo di regressione, non viene mostrato nel normale avvio.

Tre comandi IPC leggono le preferenze, salvano una configurazione e modificano la pausa. Le richieste attraversano un canale Rust e vengono eseguite sul thread del motore, evitando accessi incrociati a HWND e allo stato locale. Le configurazioni sono validate in Rust; il salvataggio atomico precede la modifica dello stato attivo. Gli eventi `accento-state` sincronizzano la UI con la pausa dal menu tray senza cancellare una bozza aperta.

La finestra React viene creata all'apertura delle impostazioni e distrutta alla chiusura. Tauri impedisce che la chiusura dell'ultima finestra termini il motore; il menu tray e un secondo avvio riaprono le impostazioni. Una bozza non salvata richiede una scelta in un dialogo con focus contenuto. I permessi web riguardano eventi e chiusura della propria finestra, non filesystem o shell. La UI distribuita è incorporata e non richiede un server locale né accesso Internet.

## Lingue

L'interfaccia include italiano, inglese, albanese, francese, spagnolo e tedesco. I messaggi Rust e React condividono `i18n.rs`; `scripts/sync-ui-messages.mjs` genera e controlla il dizionario della UI durante la build. Le traduzioni nuove richiedono una revisione linguistica editoriale: non è stata svolta una revisione da madrelingua.

I set includono italiano, albanese, francese, spagnolo, tedesco, portoghese, rumeno, polacco e turco. Sono cataloghi limitati di caratteri precomposti, non mappe linguistiche complete. L'inglese è una lingua dell'interfaccia: non serve attivare un set inglese per usare gli accenti di altri set.

L'aggiunta dei set non amplia automaticamente i layout tastiera ammessi: restano quelli latini già riconosciuti dal motore Windows. I caratteri dei nuovi set si possono usare su quei layout; non si dichiara un collaudo dei layout nazionali albanese, portoghese, rumeno, polacco o turco. Linux e macOS richiedono ancora integrazioni native dedicate.

## Libreria e dipendenze riproducibili

Lo sviluppo della presentazione condivisa è isolato nel checkout `C:/PROJECTS/Github/DimaLabs.DesignSystem-accento-flat`, branch `codex/accento-flat-product`, per preservare le modifiche già presenti nel checkout principale della libreria. Il nuovo pacchetto è `@dimalabs/react-component-flat-product@0.1.1`; compone `@dimalabs/react-component-primitives@0.2.0`.

Accento usa gli archivi npm inclusi in `vendor/` e fissati dal lockfile. Non dipende da percorsi assoluti o da una pubblicazione npm della libreria. La rigenerazione consiste in `npm run flat:build` nella libreria, `npm pack` dei pacchetti in vendor e installazione esplicita dell'archivio aggiornato nel consumatore.

## Verifiche

- 31 test Rust: motore, cataloghi, preferenze, traduzioni e rendering.
- 6 scenari Playwright: selettore ricercabile, tastiera, cambi lingua, set multipli, validazione, applicazioni escluse, temi e formato compatto. Audit automatico WCAG A/AA nei due temi: nessuna violazione serious/critical nei flussi verificati.
- 8 controlli desktop su Tauri/WebView2 reale: avvio, IPC, rifiuto di configurazioni invalide, set e lingue, persistenza, eventi, chiusura con bozza e permanenza del backend dopo la chiusura della finestra.
- Contratti SSR, controllo tipi e copertura token della libreria; catalogo Flat Product provato nel browser con TextBox, Combobox, LanguageSelector e due temi.

Il test desktop usa `--desktop-test`, una mutex distinta, nessun hook di input e una cartella temporanea delle preferenze. Non interferisce con l'istanza personale in esecuzione. Le catture browser contrassegnate “Anteprima” usano dati temporanei; `desktop-albanian-dark.png` proviene dalla finestra WebView2 reale. La prova release verifica che la pagina provenga dalle risorse incorporate.

La digitazione sintetica del collaudo Win32 completo `--self-test` non è stata rieseguita durante questa migrazione. Narrator e layout nazionali aggiuntivi restano da collaudare. Il vecchio collaudo `--ui-test` è una regressione della UI nativa conservata, non una prova della UI React.

Fonti tecniche: [comandi Tauri](https://v2.tauri.app/develop/calling-rust/), [WebviewWindowBuilder](https://docs.rs/tauri/latest/tauri/webview/struct.WebviewWindowBuilder.html). I contratti effettivi sono verificati contro le versioni fissate nei lockfile.

## Aggiornamento 0.6 · Europa e scuro fisso

Aspetto e il collegamento ai set attivi sono rimossi da Generale. La UI è sempre scura, anche con sistema chiaro; il backend normalizza a scuro le vecchie preferenze e i salvataggi IPC. Il campo schema del tema è mantenuto per compatibilità e per il collaudo nativo storico.

Il registro è ora unico per 47 voci di UI, catalogo e guide; [matrice corrente](copertura-europa.md), [fonti](copertura-europa-fonti.md). Il motore usa candidati String completi, inclusi digrammi, segni combinanti e maiuscole Unicode composte. Riconosce le lettere tradotte dal layout Windows invece dei nomi ASCII dei tasti virtuali. La libreria Flat Product 0.1.1 elimina uno scorrimento annidato nel menu ricercabile, verificato con la lista nazionale completa. I conteggi e i temi della sezione di verifica precedente descrivono la migrazione 0.5; il rapporto corrente è [verifiche-0.6.0.md](verifiche-0.6.0.md).
