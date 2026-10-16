# Verifiche di Accento 0.3.1

Aggiornamento del 16 ottobre 2026: il primo candidato viene selezionato quando scade il timer e compare il popup. Il rilascio della lettera lo conferma senza navigazione. Spazio e Destra avanzano dal primo al secondo candidato; Sinistra passa all'ultimo. Esc e i cambi di contesto continuano ad annullare. Il rilascio anticipato prima dell'apertura conserva il comportamento della falsa partenza.

La selezione avviene all'apertura, non durante l'attesa. Il rilascio del solo trigger non conferma né avanza; le ripetizioni automatiche non navigano.

Verifiche: 26 test Rust (17 motore, 9 preferenze/layout/materiale), formattazione, Clippy con warning come errori e build release superati. Le nuove regressioni coprono la prima selezione visibile, la conferma senza navigazione, il rilascio di Spazio dopo l'apertura, avanzamento e ritorno all'inizio, oltre a maiuscole, Esc e falsa partenza. Il collaudo Windows ha superato l'inserimento di `è` ed `È` senza frecce, Esc, falsa partenza, ulteriore Spazio verso `é` e Sinistra verso l'ultimo candidato. La suite completa di 35 controlli non è superata in questa sessione: ha perso il focus del destinatario durante le prove successive, e i controlli rimanenti non sono considerati verificati. Il rapporto originale è conservato in `evidenze/0.3.1/windows-self-test.txt`.

Il pacchetto è `dist/Accento-0.3.1-windows-x64.zip`. Le prove Windows usano finestre possedute dal collaudo e non salvano le preferenze reali. Restano i limiti descritti nel [rapporto 0.3.0](verifiche-0.3.0.md); le sue catture sono evidenze storiche della versione precedente.
