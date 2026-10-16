# Accento / Interfacce finali · Liquid Glass

Revisione del **16 ottobre 2026**, design e runtime **0.4.0**. Apri `Accento-Design-System.figcraft` e parti da **00 / Prodotto finale**. Le pagine 00–07 contengono testi, vettori, componenti e istanze modificabili. La pagina **08 / Prodotto implementato** incorpora **19 catture reali** dell’eseguibile verificato, comprese le interfacce in quattro lingue e le superfici del popup con alpha.

## Interfacce

- **Generale:** ritardo, set attivi, collegamento al dettaglio, aspetto e lingua dell’interfaccia indipendente dai set.
- **Set caratteri:** quattro raccolte con anteprima e selezione multipla; almeno una resta attiva.
- **BlackList:** aggiunta di un eseguibile per volta e una riga per applicazione, con rimozione esplicita.
- **Navigazione:** interruttore per spegnere e riaccendere Accento, sidebar sul desktop e tab superiori nel formato compatto.
- **Popup:** solo tasti Liquid Glass indipendenti, senza pannello comune o label. L’eventuale superficie opaca documenta l’adattamento alle preferenze di accessibilità di Windows, non un altro stile selezionabile.

La tavola standard è 1000 × 800 DIP, inclusa la barra del titolo; il client Windows è 1000 × 760. Le tre pagine compatte sono 720 × 820. Il footer e la navigazione restano fissi.

## Contenuto

| Pagina | Contenuto |
| --- | --- |
| 00 / Prodotto finale | Generale Light/Dark, popup durante la scrittura, glass su tre sfondi |
| 01 / Popup trasparente | Apertura, selezione, maiuscole, set combinati e adattamento accessibilità Windows |
| 02 / Impostazioni finali | Generale, Set caratteri e BlackList, Light/Dark e formato compatto |
| 03 / Stati e feedback | Bozza, errore, salvataggio e Accento spento |
| 04 / Componenti | Candidati, navigazione, switch, pulsanti, campi e righe dei set |
| 05 / Materiale e token | Palette, tipografia e geometria |
| 06 / Interazione e consegna | Contratto del gesto e corrispondenza con il runtime |
| 07 / Area di notifica | Menu attivo/in pausa |
| 08 / Prodotto implementato | 15 finestre reali e 4 superfici native del popup con alpha |

**9 pagine, 53 tavole, 28 componenti principali, 7 set, 41 variabili e 21 stili tipografici.** Il prototipo contiene 60 collegamenti verificati: navigazione, aspetto, interruttore e un percorso dimostrativo errore → correzione → salvataggio. Non esegue hook, digitazione o persistenza; i controlli senza collegamenti sono riferimenti visivi.

## File e rigenerazione

`accento.manifest.json` elenca tavole e componenti; `accento.inspection.json` descrive il file riaperto; `verification.json` registra hash, dimensioni, alpha e destinazioni dei collegamenti. `handoff.json` descrive il runtime. Token e variabili sono nei rispettivi JSON. `previews/` contiene gli export del documento salvato e riaperto.

Da Accento:

```powershell
./scripts/build.ps1 -Test
python -X utf8 scripts/export-ui-evidence.py
python -X utf8 scripts/create-figcraft-system.py
```

Richiede FigCraft e Pillow. La CLI locale è configurabile con `FIGCRAFT_CLI`. Il generatore aggiorna gli output generati; eventuali modifiche manuali vanno conservate in una copia prima della rigenerazione. Le catture provengono da `docs/evidenze/0.4.0/build.json`, che identifica l’hash dell’eseguibile. Per ripetere solo la verifica del documento: `python -X utf8 scripts/figcraft_verify.py`.

Il materiale Windows usa uno sfondo campionato all’apertura, sfocatura, rifrazione e alpha per pixel; non usa il motore ottico di FigCraft. Il collaudo supera 29 test Rust, 32 controlli Windows e la regressione del ridisegno; accessibilità completa e applicazioni esterne restano nel perimetro di verifica descritto in `docs/verifiche-0.4.0.md`.

La prima revisione precedente al Liquid Glass è conservata in `../archive/figcraft-before-liquid-glass.zip`; i rapporti delle versioni precedenti restano storici.
