# Usare Accento

Accento 0.3.0 è un prototipo portabile per Windows 11 x64. Inserisce accenti con tasti Glass indipendenti su fondo trasparente, lasciando il focus nel programma in cui scrivi. Non richiede PowerToys, .NET o WebView.

## Avvio

Estrai il pacchetto e apri `Accento.exe`. Compare la finestra delle impostazioni e viene aggiunta un'icona nell'area di notifica. Chiudere le impostazioni lascia attivo Accento; dal menu dell'icona puoi scegliere Pausa, Riprendi, Impostazioni ed Esci. Un secondo avvio riapre le impostazioni dell'istanza già presente.

Se stai usando anche Quick Accent o un'altra utility con lo stesso gesto, mettila in pausa durante la prova per evitare sovrapposizioni.

## Inserire un accento

1. Premi `e` e tienila premuta.
2. Premi Spazio e rilascialo, continuando a tenere premuta `e`.
3. Quando appare la barra, premi Destra per scegliere `è`, oppure Sinistra per scegliere `é`. Un'altra pressione di Spazio avanza nella lista.
4. Rilascia `e` per inserire la variante.

Per una maiuscola, inizia con Shift e la lettera, oppure con Caps Lock. Esc chiude la barra conservando la lettera originale. Il rilascio senza scelta conserva la base. Se termini prima dell'apertura, Accento restituisce lo Spazio della digitazione ordinaria.

La prima pressione di Spazio apre la barra senza selezionare. Per navigare con Spazio occorre rilasciarlo e premerlo di nuovo. Le ripetizioni automatiche non scorrono la lista. Un cambio di finestra o campo, un click, una rotellina o una scorciatoia incompatibile annulla il gesto.

## Preferenze

Le impostazioni sono divise in **Scrittura, Lingue, Aspetto e Applicazioni**. Il campo di prova in Scrittura consente di sperimentare il gesto. Premi **Salva modifiche** per applicare e conservare le modifiche; il pulsante resta disattivato finché non cambia una preferenza. Le lingue sono righe con selezione multipla, i temi formano un gruppo esclusivo e il materiale ha un interruttore distinto con anteprima. Il tema cambia subito nelle impostazioni. Il footer segnala modifiche pendenti, salvataggio o errore; un dato non valido resta nel campo e Salva riporta alla sezione da correggere.

Il ritardo si scrive nel campo oppure si regola con i pulsanti − e + a passi di 50 ms. Tab attraversa i controlli, Spazio attiva il pulsante focalizzato e le frecce cambiano sezione quando la navigazione ha il focus. Nella finestra compatta le sezioni diventano tab superiori e il testo conserva la sua dimensione. Scorre solo il contenuto: navigazione e Salva restano visibili. Il focus porta in vista i campi fuori dal viewport.

| Preferenza | Comportamento |
| --- | --- |
| Ritardo | Da 100 a 1500 ms, inizialmente 300 ms dal primo Spazio |
| Lingue | Italiano, francese, spagnolo e tedesco; almeno una selezionata |
| Tema | Automatico, Chiaro o Scuro; anteprima nelle impostazioni e applicazione al selettore dopo il salvataggio |
| Liquid Glass | Tasti con sfocatura e rifrazione del piccolo rettangolo sottostante, campionato all'apertura e tenuto solo in memoria. Tinta, bordi e alpha sono composti per tasto; nessun pannello fra i caratteri. La superficie solida segue anche contrasto elevato e trasparenza di sistema |
| Esclusioni | Nomi completi degli eseguibili separati da virgola, per esempio `notepad.exe, game.exe`; confronto esatto senza distinzione fra maiuscole e minuscole |

Il catalogo comprende varianti comuni e precomposte, con ordine stabile e senza duplicati. Non è una mappa completa di tutti i caratteri delle lingue. L'italiano include `à`, `è`, `é`, `ì`, `ò`, `ó`, `ù`; gli altri set aggiungono per esempio `ç`, `ñ`, `ü`, `ß` e le maiuscole pertinenti.

Le preferenze sono in `%LOCALAPPDATA%\Accento\settings.json`. Un file non valido rimane conservato finché non salvi una nuova configurazione. Accento non registra frasi, cronologia dei tasti o contenuti dei campi. Non usa servizi di rete.

## Requisiti e limiti della prima versione

L'ambiente iniziale è Windows 11 x64, build 22621 o successiva. L'eseguibile è stato provato sulla build 26300; la build minima non ha ancora una verifica dedicata. Il materiale è un adattamento Windows con composizione alpha per pixel, non l'effetto Apple. La rifrazione usa lo sfondo campionato all'apertura, senza aggiornamenti continui del campione durante il gesto. Con trasparenza disattivata non viene campionato lo sfondo.

Sono ammessi layout latini italiani, US/UK, francesi, tedeschi e spagnoli riconosciuti. Altri layout non attivano il selettore. Per tasti morti e combinazioni incompatibili l'attivazione viene evitata; non si dichiara supporto generale per IME, desktop remoto o composizioni complesse.

La sostituzione usa Backspace seguito da input Unicode nello stesso batch. Il collaudo automatico riguarda campi Edit Windows creati appositamente; Word, browser e terminali richiedono prove specifiche. Un cambio di testo o cursore non osservabile può ancora influire sulla sostituzione. Le applicazioni elevate possono rifiutare l'inserimento da un processo normale. Accento non richiede privilegi amministrativi.

Il riconoscimento dei campi password è limitato ai controlli Edit che espongono lo stile protetto; non copre tutti i campi dei browser. Il selettore espone un nome accessibile con candidati e scelta e invia notifiche UI Automation quando un client ascolta. La verifica completa con Narrator rimane da eseguire.

La prima consegna non include installer, avvio automatico, aggiornamenti automatici, firma, supporto ARM64 o pressione prolungata senza Spazio.

## Comandi facoltativi

`Accento.exe --background` avvia direttamente nell'area di notifica. `Accento.exe --self-test` apre una finestra di collaudo, invia sequenze sintetiche soltanto per la prova e termina; durante il collaudo lascia libera la tastiera. Il rapporto `accento-self-test.txt` e le catture della sola interfaccia di prova sono salvati accanto all'eseguibile.

Per rimuovere la versione portabile, scegli Esci e cancella la cartella estratta. Puoi conservare o rimuovere separatamente la cartella delle preferenze.
