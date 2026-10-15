# Architettura proposta di Accento

Accento separa le regole di selezione dalle API Windows e dal rendering. Il motore deve poter essere verificato con sequenze di eventi senza installare un hook globale o aprire finestre.

## Responsabilità

| Parte | Responsabilità | Confine |
| --- | --- | --- |
| Motore Rust | Stati, attivazione, candidati, navigazione, conferma e annullamento | Non chiama API Windows e non disegna finestre |
| Catalogo caratteri | Set, ordinamento e deduplicazione | Non dipende dalla finestra attiva |
| Integrazione Windows | Eventi, layout, contesto destinatario e invio Unicode | Non decide palette o disposizione dei controlli |
| Selettore | Presentazione dello stato e della selezione | Non mantiene una seconda selezione indipendente |
| Preferenze | Validazione, salvataggio locale e migrazione della configurazione | Non conserva testo digitato |
| Applicazione | Ciclo di vita, area di notifica, pausa e apertura delle impostazioni | Coordina le parti senza duplicare il motore |

Questi sono confini logici. Numero di crate e struttura delle cartelle saranno scelti durante il prototipo, evitando pacchetti separati privi di un'utilità concreta.

Il [piano derivato dall'analisi Quick Accent](piano-ricreazione-rust.md#architettura-da-prototipare) propone inizialmente due crate: `accento-core` per regole e catalogo verificabili senza Windows e `accento-windows` per l'eseguibile e le integrazioni. Il piano dettaglia anche generazioni delle sessioni, ordine degli effetti di inserimento e separazione dal rendering. Questa organizzazione è ancora da prototipare.

## Stati del motore

La sequenza proposta comprende inattivo, lettera candidata, selettore aperto e completamento o annullamento. Pausa e applicazione esclusa impediscono di iniziare una sessione. Ogni sessione conserva il contesto destinatario necessario a verificare che la conferma appartenga ancora alla stessa finestra.

L'identificativo della finestra non garantisce che cursore e contenuto siano rimasti invariati. Le politiche di annullamento riducono il rischio ma la sostituzione richiede una prova reale, soprattutto con testo selezionato, ripetizione automatica e destinatari lenti.

## Integrazione Windows da sperimentare

La prima opzione è un hook di tastiera a basso livello e un percorso di inserimento Unicode tramite le API Windows, accessibili da Rust attraverso il progetto Microsoft [windows-rs](https://github.com/microsoft/windows-rs).

Il callback dell'hook deve compiere lavoro minimo e non attendere rendering, disco o rete. Il thread che installa l'hook deve gestire il ciclo dei messaggi. Il lavoro grafico e le preferenze vanno eseguiti fuori dal callback. Sono vincoli descritti nella documentazione [LowLevelKeyboardProc](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc).

Con [SendInput](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput), l'inserimento è soggetto ai livelli di integrità Windows. Un'app non elevata non può presumere di inserire testo in un programma elevato. Il prodotto non partirà imponendo l'esecuzione come amministratore; il supporto dei contesti elevati richiede una decisione separata.

Il prototipo deve controllare il numero di eventi inseriti, evitare ricorsioni e distinguere l'input proprio senza ignorare indiscriminatamente quello di strumenti assistivi o altre utility.

## Scelta della tecnologia grafica

| Opzione | Beneficio atteso | Costo o prova necessaria |
| --- | --- | --- |
| Finestra e rendering nativi controllati da Rust | Gestione diretta del selettore e integrazione Windows | Accessibilità, testo, materiali e controlli richiedono lavoro dedicato |
| Tauri con interfaccia web | Riutilizzo possibile dei componenti React e degli strumenti DimaLabs | Focus, trasparenza della finestra, consumo della WebView e ciclo di vita da misurare |
| Selettore nativo e impostazioni web | Adatta ogni superficie al proprio compito | Aumenta dipendenze, distribuzione e complessità di verifica |

La preferenza iniziale è sperimentare il selettore nativo. Non è ancora scelto un framework grafico. Tauri rimane un'alternativa: il suo modello combina un nucleo Rust con la WebView di sistema, come descritto nella [documentazione ufficiale](https://v2.tauri.app/start/).

## Configurazione e risorse

La configurazione deve avere una versione di schema, valori predefiniti e gestione dei file non validi. Il salvataggio deve evitare di lasciare file incompleti. Le esclusioni devono avere una semantica esplicita; la prima proposta confronta il nome completo dell'eseguibile, senza corrispondenze parziali nascoste.

Il processo rimane inattivo quando non serve: nessuna animazione continua e nessuna scansione periodica del testo. RAM, CPU, dimensione installata e latenza devono essere misurate prima di pubblicare obiettivi o confronti.

