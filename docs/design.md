> **Stato corrente:** impostazioni React/Tauri con componenti e stili locali di Accento, sempre scure. La navigazione comprende Generale e BlackList. Generale riunisce ritardo e lingua sulla stessa riga e il selettore dei caratteri subito sotto, senza pagina Set caratteri separata, intestazioni decorative o riquadro guida. Il selettore conserva struttura compatta a checkbox, ricerca, set speciali, espansione e selezione complessiva, usando esclusivamente palette neutra e controlli di Accento. Il catalogo europeo e le guide seguono [copertura-europa.md](copertura-europa.md). I riferimenti precedenti descrivono le fasi storiche del progetto.

# Direzione visiva di Accento

La fonte autorevole è [Accento-Design-System.figcraft](../design/figcraft/Accento-Design-System.figcraft), revisione **design 0.4** del 16 ottobre 2026. La prima pagina mostra le interfacce complete editabili, seguite dai componenti e dalle regole. Il runtime Rust 0.4 implementa il design; la pagina 08 contiene le catture reali delle impostazioni e del popup. [Verifiche e limiti dell'implementazione](verifiche-0.4.0.md).

## Popup: solo i caratteri

La finestra del selettore è trasparente. Non contiene una superficie comune, un'intestazione, la dicitura «Varianti di», istruzioni da tastiera, nomi Unicode o codici. Le istruzioni vivono nella pagina Generale delle impostazioni.

Ogni candidato ha un tasto glass indipendente di **64 × 64 DIP**, raggio **17**, con distanza **8**. Il margine tecnico è 8: l'altezza complessiva è **80 DIP** e la larghezza è **72n + 8**, dove n è il numero di candidati. Anche con trasparenza ridotta resta vuoto lo spazio fra i tasti: cambia solo la superficie del singolo candidato.

Dal 16 ottobre 2026, all'apertura il primo candidato è già selezionato: rilasciare la lettera lo conferma senza un secondo Spazio. Spazio e Destra avanzano al successivo, Sinistra torna al precedente con ritorno all'altro estremo; Esc annulla. La selezione aggiunge tinta azzurra, bordo riconoscibile e colore del glifo. Il punto inferiore è stato rimosso. La posizione rimane in basso al centro dell'area utile del monitor attivo, come nel contratto corrente. Il focus resta nel programma destinatario.

Il FigCraft mostra apertura, scelta, maiuscole, lingue combinate, candidato singolo e superficie solida. Le esportazioni dedicate del popup hanno un vero canale alpha. I contesti di scrittura sono scene editabili di riferimento, non catture del runtime.

## Materiale

I tasti usano l'effetto `glass` nativo di FigCraft: depth 12, refraction 0,65, dispersion 0,16, frost 3, angolo luce −48°, intensità 0,72 e splay 0,1. Tre stop traslucidi, bordo luminoso, bordo interno e ombra corta completano il materiale. La tavola su tre sfondi permette di valutare luce, contrasto e trasparenza senza introdurre un pannello comune.

Le impostazioni usano superfici quiete, separatori sottili e una navigazione laterale. Il materiale flottante rimane sui candidati: la leggibilità dei campi e la gerarchia dei contenuti non dipendono da riflessi decorativi.

Il runtime Windows usa una finestra layered con alpha per pixel e senza backdrop DWM sulla radice. Il piccolo rettangolo sottostante viene campionato una volta all'apertura, sfocato e rifratto solo nei singoli tasti; il campione resta in memoria fino alla chiusura del gesto. In modalità solida il campionamento è saltato. L'adattamento mantiene geometria, contenuti e trasparenze del design, ma non usa il motore ottico di FigCraft e non aggiorna continuamente il campione. Le prove controllano anche l'alpha effettivamente disegnato da Direct2D.

Correzione del 16 ottobre 2026: la tinta del runtime è ridotta al 14% nel tema scuro e al 9% nel chiaro, con riflesso diagonale e ombre più leggere. Il backdrop viene diffuso con tre passaggi di sfocatura, raggio 6 DIP, e ricomposto con alpha di base 250/255: la trasparenza visiva viene dal campione sfocato, evitando che il testo nitido sottostante attraversi il materiale. Il bordo glass è un riflesso sottile più luminoso verso l'alto a sinistra, senza doppio contorno uniforme. Per evitare tasti grigi pieni su applicazioni chiare, il glass conserva soprattutto il colore dello sfondo. Il colore del glifo segue la luminanza del materiale composto sotto ciascun candidato; la scelta resta riconoscibile tramite tinta e bordo azzurri, con glifo a contrasto. L'anteprima delle impostazioni usa gradienti traslucidi. L'alternativa solida conserva il comportamento precedente.

## Impostazioni

La tavola standard è **1000 × 800 DIP**, con sidebar di **216 DIP**. Il client Windows iniziale è 1000 × 760 DIP, più la barra del titolo nativa. Il contenuto parte a 248 DIP; controlli alti 40–44 DIP e footer stabile. Le destinazioni sono tre:

1. **Generale:** ritardo di apertura, set attivi in forma compatta, collegamento al dettaglio, Sistema/Chiaro/Scuro e lingua dell’interfaccia indipendente dai set.
2. **Set caratteri:** quattro righe con nome, caratteri rappresentativi e selezione multipla. Almeno un set resta attivo.
3. **BlackList:** campo per un singolo eseguibile, Aggiungi e una riga per applicazione con rimozione esplicita. Validazione accanto al campo.

L’interruttore attivo/spento rimane nella navigazione ed è immediato, senza salvataggio. Le altre preferenze conservano Salva modifiche. Il popup ha un solo materiale Liquid Glass; non esiste più una preferenza di trasparenza. L’eventuale superficie opaca è un adattamento alle impostazioni di accessibilità di Windows.

Sotto 900 DIP le tre sezioni diventano tab superiori e l’interruttore resta in alto. Scorre il solo contenuto, tagliato da un viewport nativo; navigazione e footer non si muovono. La barra sottile occupa un margine riservato e non cambia la larghezza del contenuto. Rotella e focus aggiornano la geometria solo quando cambia davvero lo scorrimento. Clic sui set e modifiche dei valori non riposizionano i controlli. Ogni superficie viene composta fuori schermo e mostrata con una sola copia, evitando fotogrammi intermedi vuoti.

## Tipografia, accessibilità e interazione

Segoe UI Variable: titolo 28, riga 14, supporto 12/13, glifo 32. Lo spazio segue multipli di 4 e 8. Focus esterno di 2 DIP e stati di hover distinti. Le icone sono vettori originali; non sono richiesti font Apple.

Restano invariati il gesto, la deduplicazione del catalogo e i vincoli delle preferenze. Esc annulla, il rilascio della lettera conferma, una nuova pressione volontaria di Spazio o le frecce naviga. Queste regole sono spiegate nelle impostazioni e nella consegna tecnica, senza aggiungere testo al popup.

Il prototipo FigCraft collega sezioni, tema, interruttore e un esempio di correzione/salvataggio. Le selezioni mostrate sono stati progettati, non una simulazione degli hook o della persistenza. Le prove con tastiera, Narrator, scala del testo, DPI differenti, contrasto elevato e trasparenza disattivata restano parte della verifica del prodotto implementato.

[Inventario, anteprime e rigenerazione](../design/figcraft/README.md). [Corrispondenza con l'implementazione](../design/figcraft/handoff.json). La versione precedente è conservata nell'archivio del design; i rapporti del runtime 0.2 restano storici e non descrivono questa nuova UI.
