# Fonti e riferimenti di Accento

Questi riferimenti sostengono la definizione e l'analisi del progetto. Le capacità documentate da altre applicazioni e piattaforme rimangono distinte dalle funzioni che saranno implementate e provate in Accento.

## Quick Accent

- [Microsoft Learn in italiano](https://learn.microsoft.com/it-it/windows/powertoys/quick-accent): riferimento funzionale per attivazione, set di caratteri e impostazioni.
- [Sorgente della documentazione](https://github.com/MicrosoftDocs/windows-dev-docs/blob/docs/hub/powertoys/quick-accent.md): utile per la provenienza del testo; il contenuto consultato nella prima analisi differiva dalla pagina Learn, in particolare per la pressione prolungata.
- [Repository PowerToys](https://github.com/microsoft/PowerToys): riferimento dell'implementazione. La versione fissata e il perimetro dell'analisi sono riportati nella sezione seguente.

Per confronti futuri fissare una versione o un commit: i collegamenti ai rami correnti possono cambiare. L'ispirazione funzionale non autorizza a copiare asset, dati o codice senza verificarne le condizioni d'uso.

### Versione analizzata con REA

L'[analisi del 15 ottobre 2026](reverse-engineering-quick-accent.md) usa i binari locali 0.101.2362.0 e il tag sorgente `v0.101.2362.0`, risolto al commit [`389b1a3827b2cf577723288df15437490ef0663a`](https://github.com/microsoft/PowerToys/commit/389b1a3827b2cf577723288df15437490ef0663a). Il [manifest](evidenze/quick-accent/manifest.json) registra hash e URL; il [bundle REA](evidenze/quick-accent/rea-bundle.json) conserva i record originali. Metadati e CIL installati, lettura del sorgente e prove runtime sono evidenze distinte. La parte nativa non è stata decompilata.

## API e strumenti

- [LowLevelKeyboardProc](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc): callback, ciclo dei messaggi e vincoli temporali degli hook.
- [SendInput](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput): inserimento sintetico, valore restituito e limiti dei livelli di integrità.
- [Rust for Windows](https://github.com/microsoft/windows-rs): accesso alle API Windows da Rust; crate e versioni vanno selezionati al momento dell'implementazione.
- [Tauri](https://v2.tauri.app/start/): alternativa con nucleo Rust e interfaccia web nella WebView di sistema.
- [Apple Materials](https://developer.apple.com/design/human-interface-guidelines/materials): riferimento del materiale e dei suoi ruoli, da leggere insieme alla ricerca locale.
- [DWM system backdrop](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/ne-dwmapi-dwm_systembackdrop_type): materiale per la prova nativa Glass e requisito di build dell'API.
- [Stili estesi delle finestre](https://learn.microsoft.com/en-us/windows/win32/winmsg/extended-window-styles): base per sperimentare il selettore non attivante.

Queste fonti sono state utilizzate nell'analisi preliminare della conversazione del 15 ottobre 2026. La disponibilità delle API non prova la fattibilità di tutti i comportamenti proposti.

## Ricerca DimaLabs

- [Ricerca Apple moderno](../../DimaLabs.DesignSystem/docs/ricerca-apple-design-moderno.md): principi, piattaforme, materiali e riferimenti.
- [Traduzione pratica](../../DimaLabs.DesignSystem/docs/apple-glass-pratica-web.md): separazione fra API native, composizione e adattamento web.
- [Caso Apple Glass](../../DimaLabs.DesignSystem/docs/cases/apple-glass.md): implementazione del catalogo e limiti delle sue verifiche.

I collegamenti locali presumono che Accento e DimaLabs.DesignSystem siano checkout affiancati. Il progetto Accento non dipende da quei file per essere eseguito e non eredita automaticamente l'implementazione o gli esiti dei test del catalogo.

## Manutenzione

Quando una fonte determina una scelta tecnica, registrarne la versione o revisione nel documento della decisione. Conservare le motivazioni delle scelte superate e aggiornare requisiti, design e piano insieme quando cambia il comportamento del prodotto.

