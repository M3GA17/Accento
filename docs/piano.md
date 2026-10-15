# Piano di sviluppo e verifica di Accento

Lo sviluppo procede dimostrando prima l'affidabilità dell'inserimento, poi la qualità del selettore e infine la distribuzione. Le fasi seguenti descrivono lavoro futuro; non hanno scadenze o risultati già acquisiti.

L'[analisi REA di Quick Accent](reverse-engineering-quick-accent.md) identifica la versione installata 0.101.2362.0 e i punti delicati del flusso. Il [piano di ricreazione Rust](piano-ricreazione-rust.md) dettaglia queste fasi con il contratto proposto del gesto, le consegne R0–R5 e l'esperimento Glass nativo.

## Fasi e condizioni di uscita

| Fase | Consegna | Condizione per procedere |
| --- | --- | --- |
| 1 Prova del motore | Modello degli stati e sequenze di eventi verificabili | Attivazione, annullamento, maiuscole e navigazione hanno esiti deterministici |
| 2 Prova Windows | Selettore essenziale e inserimento nel destinatario | Focus conservato e casi ordinari senza perdita o duplicazione di caratteri negli scenari provati |
| 3 Scelta grafica | Confronto delle opzioni e prototipo Glass | Materiale leggibile, comportamento accessibile da valutare e consumi misurati |
| 4 Prima versione utilizzabile | Preferenze, set linguistici, esclusioni e pausa | I flussi principali funzionano dopo chiusura e riavvio |
| 5 Preparazione della distribuzione | Pacchetto, istruzioni e matrice di compatibilità | Installazione, aggiornamento e rimozione verificati sul sistema minimo dichiarato |

La prova Windows può portare a rivedere la semantica di sostituzione prima di investire nel rendering. Un mockup può esplorare l'aspetto, ma non chiude quella verifica.

## Matrice iniziale delle prove

| Scenario | Cosa controllare |
| --- | --- |
| Blocco note | Sequenza base, ripetizione, Esc e rilascio senza selezione |
| Browser | Input, textarea, editor contenteditable e cambi di focus |
| Word | Sostituzione vicino a formattazione, selezioni e autocorrezione |
| VS Code | Editor, scorciatoie e terminale integrato come contesti distinti |
| Windows Terminal | Compatibilità dell'inserimento e assenza di caratteri di controllo inattesi |
| Layout italiano e US | Shift, Caps Lock, scorciatoie e caratteri di partenza |
| Layout con AltGr e tasti morti | Conservazione del comportamento originale e disattivazione nei casi non supportati |
| IME | Gestione sicura della composizione o esclusione dichiarata |
| Cambio finestra durante la scelta | Nessun invio della conferma al nuovo destinatario |
| Due monitor | Posizione, area utile e scale 100, 150 e 200 per cento |
| Aspetto e accessibilità | Chiaro, scuro, opaco, contrasto elevato, movimento ridotto e Narrator |
| Applicazione elevata | Identificazione e comunicazione del limite di inserimento |
| Pausa e uscita | Interruzione della sessione e nessun tasto lasciato in uno stato incoerente |

## Misure da raccogliere

Registrare CPU a riposo, memoria del processo e degli eventuali processi WebView, tempo fra attivazione e primo frame, tempo di conferma, dimensione del pacchetto e tempo di avvio. Le misure devono riportare hardware, versione Windows, build dell'app, carico di lavoro e numero di campioni.

Un confronto con Quick Accent usa la stessa macchina, gli stessi programmi e una versione identificata di PowerToys. Distinguere il costo della suite dal costo attribuibile al modulo: il confronto fra processi diversi non dimostra da solo l'efficienza del motore.

## Evidenze di ogni verifica

Per ogni prova registrare revisione del codice, ambiente, passi, risultato atteso, risultato osservato ed eventuali problemi. Separare test del motore, prove manuali Windows, misure e controllo visivo. Una schermata non dimostra l'inserimento corretto o l'accessibilità.

## Decisioni ancora aperte

| Decisione | Quando affrontarla | Evidenza richiesta |
| --- | --- | --- |
| Versione minima Windows e architetture | Prima della scelta grafica definitiva | API, rendering e disponibilità degli ambienti di prova |
| Framework grafico | Dopo la prova del selettore | Focus, accessibilità, leggibilità e consumi |
| Semantica finale di conferma | Dopo la prova Windows | Confronto fra gesture e casi di errore |
| Ritardo predefinito | Durante le prove di scrittura | Attivazioni involontarie e facilità di apprendimento |
| Supporto contesti elevati | Prima di dichiarare la compatibilità | Limiti di integrità e strategia di distribuzione |
| Licenza e riuso di codice o dati | Prima di incorporare risorse esterne | Provenienza e condizioni delle singole risorse |
| Installer, firma e aggiornamenti | Prima della distribuzione | Esperienza di installazione e manutenzione |
| Identità pubblica Accento | Prima della pubblicazione | Verifica di nomi, domini e marchi pertinenti |

## Stato delle verifiche

Al 15 ottobre 2026 è disponibile il prototipo 0.1.0: 16 test Rust e 18 controlli di integrazione su campi Edit Windows superati, con inserimento, mantenimento del focus, annullamento, pausa, esclusioni e tre aspetti del selettore. Il [rapporto della versione](verifiche-0.1.0.md) delimita i risultati. La matrice delle applicazioni, Narrator, DPI su più monitor e prestazioni restano da verificare; R3 e le fasi successive non sono chiuse dalla presenza dell'eseguibile.

