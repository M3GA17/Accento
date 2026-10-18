<p align="center"><img src="ui/public/brand/accento.svg" width="88" height="88" alt="Il logo Accento: una á dentro una card di vetro"></p>

<h1 align="center">Accento</h1>

<p align="center"><strong>Gli accenti, proprio dove scrivi.</strong><br>Tieni premuta una lettera. Tocca Spazio. Scegli il carattere.</p>

<p align="center"><a href="README.md">English</a> · <a href="docs/uso.md">Guida completa</a> · <a href="docs/DEVELOPMENT.md">Sviluppo e compilazione</a></p>

<p align="center"><img src="docs/assets/hero.png" width="1200" alt="Accento: tasti Glass indipendenti e impostazioni scure con il nuovo logo"></p>

## Un piccolo gesto, più caratteri

Per scrivere *caffè*, *français* o *piñata* non dovrebbe servire aprire una mappa caratteri. Accento mostra accenti e varianti in piccole card Glass, senza spostare il focus dal programma in cui stai scrivendo.

1. Tieni premuta `e`.
2. Premi e rilascia Spazio, continuando a tenere premuta la lettera.
3. Attendi il popup: `è` è già selezionata. Usa ← / → o premi di nuovo Spazio per cambiare scelta.
4. Rilascia `e` per inserire il carattere. Esc annulla.

Per le maiuscole, inizia con Shift o Caps Lock. Se termini il gesto prima dell'apertura, viene mantenuta la digitazione ordinaria.

Il set **Valute** è attivo inizialmente: sulla `e` trovi anche `€`, dopo `è` e `é`. Puoi disattivarlo in **Generale → Set caratteri → Set speciali**, dove trovi tutte le associazioni, per esempio `s` → `$`, `l` → `£`, `y` → `¥`. [Come inserire le valute](docs/uso.md#valute).

## Stato della beta

**[Scarica Accento 0.5.0-beta.3](https://github.com/M3GA17/Accento/releases/tag/v0.5.0-beta.3)** — installer e ZIP portabile Windows x64, app universale macOS per Intel e Apple Silicon e checksum SHA-256. Il repository è privato: per scaricare serve l’accesso al repository. Questa beta raccoglie il feedback sulle prove reali Mac prima della 0.5 stabile. [Note della release](docs/releases/v0.5.0-beta.3.md).

| Sistema | Stato |
| --- | --- |
| Windows 11 x64 Intel/AMD | Implementazione nativa; installer e ZIP portabile disponibili |
| Linux X11 | Backend nativo implementato, build Linux e test reali di inserimento superati |
| macOS 13+ Intel / Apple Silicon | App beta universale; controlli di build/avvio e prove manuali descritti nelle note della release |

Il motore del gesto e le preferenze sono condivisi. Windows conserva il backend Win32; Linux usa XInput2/XTEST e macOS Quartz/Accessibility. **Wayland nativo non è ancora supportato**: l’inserimento resta disabilitato con una spiegazione nelle impostazioni. [Guida alle piattaforme e alle verifiche](docs/cross-platform.md). I pacchetti Windows locali precedenti al porting rimangono build solo Windows.

La 0.5 stabile comprende Windows e macOS. Linux X11 e Wayland sono rinviati a una versione successiva.

## Pacchetti Windows

L'**installer** installa Accento per l'utente corrente e può scaricare Microsoft Edge WebView2 Runtime se manca. Per quel passaggio può servire una connessione; Accento funziona offline.

Per il **portabile**, estrai l'intero ZIP e apri `Accento.exe`, conservando guide e licenze. WebView2 deve essere già installato. Le preferenze rimangono in `%LOCALAPPDATA%\Accento\settings.json`, separate dalla cartella del programma.

Non servono PowerToys, .NET, Node.js o Rust per usare l'app. I binari beta non sono firmati e non hanno aggiornamenti automatici. I checksum dei due pacchetti sono in `SHA256SUMS.txt`.

## Beta macOS

Apri il DMG e trascina `Accento.app` in Applicazioni, oppure estrai lo ZIP universale e sposta lì l'app. La beta ha firma ad hoc e non è notarizzata: se macOS blocca il download attendibile, segui la [procedura Apple Apri comunque](https://support.apple.com/it-it/102445). Concedi Accessibilità e, se richiesto, Monitoraggio input. Esci e riapri la copia installata se necessario, poi riattiva Accento. La [guida alle prove Mac](docs/distribution/TEST-MACOS.it.txt) è inclusa nelle risorse dell'app e allegata alla release. Digitazione e flusso dei permessi attendono il riscontro del tester.

## Lingue e impostazioni

- **47 voci linguistiche e varianti scritte**, comprese scritture latina, greca, cirillica, armena e georgiana.
- Lingua dell'interfaccia e set caratteri indipendenti, con ricerca e selezione di più set.
- Candidati completi, inclusi i digrammi e i segni combinanti supportati, con le rispettive maiuscole.
- Ritardo da 100 a 1500 ms, inizialmente 300 ms.
- BlackList per nome dell'eseguibile, pausa immediata, bozza con Salva/Annulla e protezione dalla chiusura con modifiche non salvate.

<p align="center"><img src="docs/assets/settings.png" width="960" alt="Le impostazioni di Accento nell'app Windows, catturate con un profilo di esempio in inglese"></p>

Chiudere le impostazioni lascia attivo il motore; scegli Esci dall'icona nell'area di notifica per fermarlo. La [matrice di copertura](docs/copertura-europa.md) documenta repertori, fonti e 47 guide rapide. Le nuove traduzioni sono bozze ancora da revisionare con madrelingua.

## Privacy e limiti

Accento non salva frasi digitate, cronologia dei tasti o contenuti dei campi e non usa servizi di rete. Le impostazioni restano locali. Il popup campiona un piccolo sfondo all'inizio del gesto e lo conserva solo in memoria; le impostazioni di accessibilità Windows possono richiedere una superficie opaca.

È una beta: applicazioni elevate, IME complessi, sessioni remote, campi protetti e singoli layout richiedono prove dedicate. Il riconoscimento dei campi password non copre tutti i browser. Metti in pausa altre utility con lo stesso gesto durante la prova. La [guida d'uso](docs/uso.md) riporta comportamenti e limiti completi.

## Sviluppo

Rust per il motore condiviso e le integrazioni native; React, TypeScript e Tauri 2 per le impostazioni e il picker Unix. Il popup Windows resta nativo. Componenti, stili e colori sono mantenuti direttamente in `ui/src/components` e `ui/src/theme.css`; React Aria gestisce le interazioni accessibili. Non servono pacchetti DimaLabs né checkout affiancati.

```powershell
npm ci
npm run desktop
npm run bundle
```

La [guida di sviluppo](docs/DEVELOPMENT.md) descrive prerequisiti, test e struttura. La [procedura di rilascio](docs/RELEASING.md) separa preparazione locale e pubblicazione manuale: il comando di impacchettamento non pubblica nulla.

Sorgente Accento con [licenza MIT](LICENSE); le dipendenze conservano le proprie condizioni nei [relativi avvisi](THIRD-PARTY-NOTICES.md).
