# Verifiche Accento 0.4.0

16 ottobre 2026. Windows 11 x64 build 26300, Rust 1.98.0 MSVC. Versione portabile, nessuna installazione o scrittura nelle preferenze personali durante i collaudi.

## Risultato

- 29 test Rust superati: 17 del motore e 12 di layout, materiale, preferenze e traduzioni.
- Formattazione e controlli statici senza avvisi.
- 14 controlli delle impostazioni superati nella finestra reale, senza hook di input.
- Regressione del ridisegno superata: zero ridisegni della radice e zero riposizionamenti dei controlli su dieci eventi di rotella al limite; zero riposizionamenti al clic su un set.
- 32 controlli Windows completi superati, compresi i 14 delle impostazioni, prima selezione automatica, maiuscole, annullamento, cambio campo, pausa, esclusione esatta del processo, temi chiaro/scuro e alpha dei popup.
- FigCraft salvato, riaperto e verificato: 9 pagine, 53 tavole, 60 collegamenti validi. La pagina 08 contiene 19 catture di questa versione.

Il pacchetto è `dist/Accento-0.4.0-windows-x64.zip`. Il JSON [build.json](evidenze/0.4.0/build.json) identifica eseguibile, hash, ambiente e catture. I rapporti testuali sono [Windows](evidenze/0.4.0/windows-self-test.txt), [impostazioni](evidenze/0.4.0/settings-controls.txt) e [ridisegno](evidenze/0.4.0/settings-repaint.txt).

## Difetto riprodotto e correzione

La registrazione fornita mostra le impostazioni durante clic e rotella. Il feedback deterministico usa lo stesso percorso Win32 della rotella nella pagina dei set, senza modificare i dati: `cargo build -p accento-windows`, poi `target/debug/Accento.exe --ui-test`.

Prima della correzione: `FAIL: wheel at top: 10 unnecessary root paints / 10 events; preferences unchanged: true`. Dopo aver eliminato il ridisegno quando lo scorrimento non cambia: `PASS`, con zero ridisegni. Il controllo permanente verifica anche che i clic sui set non riposizionino i controlli.

La causa confermata era il richiamo incondizionato del layout e del ridisegno completo anche quando rotella e clic non cambiavano la geometria. Ora gli aggiornamenti dei valori invalidano i soli controlli coinvolti e il footer; rotella e focus aggiornano il layout solo per uno spostamento effettivo. Posizione e visibilità dei controlli vengono modificate solo quando cambiano. La barra di scorrimento è disegnata nel margine riservato: non cambia la larghezza della finestra. La composizione fuori schermo pubblica una superficie completa, evitando l’esposizione del passaggio di cancellazione.

## Configurazione e interfaccia

Generale contiene ritardo, set attivi, aspetto e lingua dell’interfaccia; Set caratteri espone le raccolte; BlackList contiene un eseguibile per riga. La lingua UI è italiana, francese, spagnola o tedesca, indipendente dai set. Il menu dell’area di notifica e i nomi accessibili seguono la lingua salvata. L’interruttore è immediato, conserva la bozza e sospende il motore; alla riapertura del programma Accento parte attivo.

Il materiale ha un solo stile Liquid Glass. La preferenza `transparency` è rimossa dal modello e dal JSON salvato; i vecchi JSON che la contengono vengono letti mantenendo ritardo, set ed esclusioni. La lingua UI mancante usa italiano. Versioni di schema non supportate restano rifiutate. L’adattamento opaco alle preferenze di accessibilità di Windows rimane disponibile al sistema.

La BlackList normalizza gli inserimenti, rifiuta duplicati senza distinzione tra maiuscole e minuscole, nomi multipli, percorsi e caratteri non ammessi. Aggiunta e rimozione restano nella bozza fino a Salva. Il test di salvataggio atomico verifica insieme lingua UI, set ed esclusioni e preserva il file esistente in caso di configurazione non valida.

## Ripetere le verifiche

```powershell
./scripts/build.ps1 -Test
python -X utf8 scripts/export-ui-evidence.py
python -X utf8 scripts/create-figcraft-system.py
```

La compilazione normale include il collaudo delle impostazioni; `-Test` aggiunge le sequenze di tastiera nella finestra di prova posseduta dal programma. Gli eventi sintetici vengono inviati solo se quella finestra conserva il primo piano. Il normale avvio non salva catture o cronologia. Le evidenze sono catture native, non mockup; le superfici alpha provengono dal compositore del popup.

## Limiti del risultato

La verifica riguarda finestre Edit Win32 possedute dal collaudo e lo stesso rendering delle impostazioni distribuite. Non certifica ogni fotogramma del compositore su altri hardware, l’inserimento in tutte le applicazioni esterne, Narrator completo, desktop remoti o configurazioni fisiche con più monitor e DPI diversi. Questi limiti non sono sostituiti dalle catture. Il motore ottico di FigCraft è un riferimento e non coincide con quello Windows.
