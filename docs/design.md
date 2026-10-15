# Direzione visiva di Accento

La fonte autorevole è [Accento-Design-System.figcraft](../design/figcraft/Accento-Design-System.figcraft), revisione **design 0.3** del 15 ottobre 2026. La prima pagina mostra le interfacce complete proposte per il prodotto finale, seguite dai componenti e dalle regole. Il runtime Rust 0.2 usa ancora il layout precedente: il lavoro parte dal FigCraft e rende esplicita la successiva consegna all'implementazione.

## Popup: solo i caratteri

La finestra del selettore è trasparente. Non contiene una superficie comune, un'intestazione, la dicitura «Varianti di», istruzioni da tastiera, nomi Unicode o codici. Le istruzioni vivono nella sezione Scrittura delle impostazioni.

Ogni candidato ha un tasto glass indipendente di **64 × 64 DIP**, raggio **17**, con distanza **8**. Il margine tecnico è 8: l'altezza complessiva è **80 DIP** e la larghezza è **72n + 8**, dove n è il numero di candidati. Anche con trasparenza ridotta resta vuoto lo spazio fra i tasti: cambia solo la superficie del singolo candidato.

All'apertura nessuna scelta è attiva. La selezione aggiunge tinta azzurra, bordo riconoscibile e colore del glifo. Il punto inferiore è stato rimosso. La posizione rimane in basso al centro dell'area utile del monitor attivo, come nel contratto corrente. Il focus resta nel programma destinatario.

Il FigCraft mostra apertura, scelta, maiuscole, lingue combinate, candidato singolo e superficie solida. Le esportazioni dedicate del popup hanno un vero canale alpha. I contesti di scrittura sono scene editabili di riferimento, non catture del runtime.

## Materiale

I tasti usano l'effetto `glass` nativo di FigCraft: depth 12, refraction 0,65, dispersion 0,16, frost 3, angolo luce −48°, intensità 0,72 e splay 0,1. Tre stop traslucidi, bordo luminoso, bordo interno e ombra corta completano il materiale. La tavola su tre sfondi permette di valutare luce, contrasto e trasparenza senza introdurre un pannello comune.

Le impostazioni usano superfici quiete, separatori sottili e una navigazione laterale. Il materiale flottante rimane sui candidati: la leggibilità dei campi e la gerarchia dei contenuti non dipendono da riflessi decorativi.

Il renderer Windows deve riprodurre questo risultato sul compositor reale. Applicare un backdrop alla radice rettangolare del popup riempirebbe anche gli spazi fra tasti e violerebbe il design. Il prossimo intervento deve quindi affrontare insieme forma della finestra, composizione e disegno dei candidati. L'effetto FigCraft non certifica una realizzazione Liquid Glass su Windows.

## Impostazioni

La finestra standard è **1000 × 740 DIP**, con sidebar di **216 DIP**, margine del contenuto di 40, campi e azioni alti 40. La gerarchia segue quattro destinazioni:

1. **Scrittura:** gesto Lettera + Spazio, ritardo, campo di prova.
2. **Lingue:** quattro righe con nomi, caratteri rappresentativi e selezione multipla. Almeno una lingua resta attiva.
3. **Aspetto:** Automatico/Chiaro/Scuro in un gruppo esclusivo; materiale del popup in una riga distinta, con anteprima reale dei candidati.
4. **Applicazioni:** nomi dei processi esclusi, formato richiesto ed errore accanto al dato.

Il footer conserva la posizione di Salva modifiche e comunica dati salvati, modifiche pendenti o errore. Il salvataggio resta esplicito. Input non valido e relativa correzione sono visibili nella pagina Stati e feedback.

La versione compatta è **720 × 820 DIP**: le quattro destinazioni diventano tab superiori, i contenuti si adattano alla larghezza e la tipografia mantiene la sua dimensione. Non viene più ridotto uniformemente tutto il canvas.

## Tipografia, accessibilità e interazione

Segoe UI Variable: titolo 28, riga 14, supporto 12/13, glifo 32. Lo spazio segue multipli di 4 e 8. Focus esterno di 2 DIP e stati di hover distinti. Le icone sono vettori originali; non sono richiesti font Apple.

Restano invariati il gesto, la deduplicazione del catalogo e i vincoli delle preferenze. Esc annulla, il rilascio della lettera conferma, una nuova pressione volontaria di Spazio o le frecce naviga. Queste regole sono spiegate nelle impostazioni e nella consegna tecnica, senza aggiungere testo al popup.

Il prototipo FigCraft collega sezioni, tema, materiale e un esempio di correzione/salvataggio. Le selezioni mostrate sono stati progettati, non una simulazione degli hook o della persistenza. Le prove con tastiera, Narrator, scala del testo, DPI differenti, contrasto elevato e trasparenza disattivata restano parte della verifica del prodotto implementato.

[Inventario, anteprime e rigenerazione](../design/figcraft/README.md). [Differenze da implementare](../design/figcraft/handoff.json). La versione precedente è conservata nell'archivio del design; i rapporti del runtime 0.2 restano storici e non descrivono questa nuova UI.
