# Accento / Interfacce finali · Liquid Glass

Apri **Accento-Design-System.figcraft** e parti da **00 / Prodotto finale**. La prima pagina mostra le finestre complete, chiare e scure, e il popup dentro un contesto di scrittura. Le UI sono costruite con testi, vettori, componenti e istanze native modificabili; non sono screenshot appiattiti.

Revisione del 15 ottobre 2026: **design e runtime 0.3**. Il design è implementato nell'eseguibile Rust. La pagina **08 / Prodotto implementato** incorpora le catture reali del collaudo Windows, incluse le superfici del popup con alpha; le pagine 00–07 conservano i layer e i componenti editabili.

## Cosa cambia

- **Popup senza sfondo:** nessun pannello esterno, bordo comune, titolo, label o istruzione. Solo caratteri dentro tasti glass indipendenti 64 × 64 DIP, gap 8, raggio 17. La radice e lo spazio fra i tasti restano trasparenti; le ombre appartengono ai singoli tasti.
- **Liquid Glass nativo:** effetto FigCraft `glass` con rifrazione, dispersione e luce. Tinte traslucide e bordi sottili; alternativa solida con la stessa geometria. Il confronto su documento chiaro, editor scuro e fondo a colori è nella prima pagina.
- **Impostazioni ordinate:** Scrittura, Lingue, Aspetto, Applicazioni. Sidebar stabile, gruppi sobri, righe lingua compatte, tema separato dal materiale, footer coerente con le modifiche pendenti.
- **Schermate di prodotto:** finestra 1000 × 740 e quattro sezioni compatte 720 × 820. La versione compatta riordina la navigazione in alto e conserva la dimensione del testo.

## Nel file

| Pagina | Contenuto |
| --- | --- |
| 00 / Prodotto finale | UI complete Light/Dark, popup durante la scrittura, glass su tre sfondi |
| 01 / Popup trasparente | Apertura, scelta, maiuscole, lingue combinate, candidato singolo, alternativa solida; export con alpha |
| 02 / Impostazioni finali | Tutte le quattro sezioni Light/Dark e tutte le quattro sezioni compatte |
| 03 / Stati e feedback | Modifiche pendenti, errore inline, salvataggio; tema esplicito e materiale on/off |
| 04 / Componenti | Candidati, navigazione, switch, pulsanti, campi e righe lingua; selezione, hover e focus |
| 05 / Materiale e token | Palette Light/Dark, trasparenze, tipografia, geometria |
| 06 / Interazione e consegna | Contratto del gesto e differenze rispetto al runtime |
| 07 / Area di notifica | Comandi attivo/in pausa; menu come riferimento per Windows |
| 08 / Prodotto implementato | 13 catture reali: impostazioni, quattro sezioni compatte, errori e superfici native con alpha |

**9 pagine, 56 tavole esportate, 28 componenti principali, 7 set, 41 variabili, 21 stili tipografici.** Le tavole comprendono 43 riferimenti di design e 13 catture del runtime. L'inventario esatto è in `accento.manifest.json`.

Nel prototipo si possono visitare le quattro sezioni, passare da Chiaro a Scuro, cambiare il materiale e percorrere un esempio di correzione e salvataggio. I percorsi sono dimostrativi: il campo errato porta al valore corretto predisposto e Salva porta allo stato salvato. Non eseguono scrittura, hook o persistenza reali. I controlli non collegati sono riferimenti di interfaccia.

## File e rigenerazione

- `Accento-Design-System.figcraft`: fonte autorevole del design.
- `previews/`: esportazioni dal file salvato e riaperto; i `popup-*.png` conservano il canale alpha.
- `accento.tokens.json` e `accento.variables.json`: materiale, colori e misure.
- `handoff.json`: modifiche implementate nei sorgenti Rust, con stato e riferimento alle evidenze.
- `accento.inspection.json`: struttura completa riaperta dall'editor.
- `verification.json`: controllo delle dimensioni, del canale alpha, dei collegamenti e dei componenti.

Da Accento: `python -X utf8 scripts/create-figcraft-system.py`. Richiede FigCraft e Pillow per la verifica dei PNG; la CLI locale è configurabile con `FIGCRAFT_CLI`. Il generatore sostituisce gli output generati: salva eventuali modifiche manuali in una copia prima di rigenerare. `scripts/figcraft_mcp.py` contiene il trasporto locale, senza modifiche a configurazioni o pubblicazione di librerie. Per ripetere soltanto i controlli: `python -X utf8 scripts/figcraft_verify.py`.

Per rinnovare le catture reali: esegui il collaudo `scripts/build.ps1 -Test`, poi `python -X utf8 scripts/export-ui-evidence.py` e rigenera il FigCraft. L'ultima pagina importa i PNG elencati in `docs/evidenze/0.3.0/build.json`, con l'hash dell'eseguibile verificato.

La revisione precedente, incluso il generatore, è conservata in `../archive/figcraft-before-liquid-glass.zip`.

## Verifica

Il generatore salva e riapre il file nativo, verifica l'inventario e controlla che i master del popup abbiano radici senza fill, stroke, effetti o testi aggiuntivi. Tutte le tavole vengono esportate. Il controllo supplementare verifica hash, dimensioni, alpha degli angoli e dello spazio fra tasti, glifi e destinazioni dei collegamenti. La revisione visiva comprende schermate Light/Dark, tutte le sezioni compatte, errore, componenti, palette, popup contestuale e confronto su tre sfondi.

La rifrazione di FigCraft è un riferimento visivo. Il runtime Windows usa un materiale proprio con sfondo campionato all'apertura, sfocatura, rifrazione e alpha per pixel; il campione non viene aggiornato continuamente durante il gesto. Il collaudo 0.3 verifica composizione, trasparenza e focus, ma non certifica accessibilità completa o fedeltà al motore ottico Apple. Le evidenze del runtime 0.2 restano in `docs/evidenze/0.2.0/`.
