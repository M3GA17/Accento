# Interazione e requisiti di Accento

Il selettore deve lasciare il focus al programma in cui la persona sta scrivendo. Apertura, navigazione e chiusura non devono spostare il cursore di testo o attivare la finestra di Accento.

## Gesto iniziale proposto

La prima modalità usa una lettera mantenuta premuta e Spazio come tasto di attivazione. L'intercettazione di Spazio avviene solo quando esistono candidati e la sequenza soddisfa le condizioni di attivazione. Negli altri casi gli eventi mantengono il comportamento ordinario.

Il ritardo sarà configurabile. Il prototipo dovrà confrontare più valori prima di stabilire quello predefinito: nessun tempo è già considerato ottimale.

## Percorso principale

1. La persona preme `e`: il programma destinatario riceve la lettera normalmente.
2. Mantenendo `e` premuta, attiva il selettore con Spazio.
3. Accento mostra le varianti del set attivo. In apertura nessuna variante è selezionata o confermata.
4. Freccia sinistra e destra spostano la selezione; ulteriori pressioni volontarie di Spazio avanzano. Il primo comando successivo all'attivazione sceglie il primo candidato con Spazio o Destra, l'ultimo con Sinistra; la navigazione ritorna all'altro estremo della lista.
5. Il rilascio della lettera base conferma il candidato, se selezionato. Il rilascio del solo Spazio non conferma.
6. Accento sostituisce la lettera base nel programma destinatario e chiude il selettore.

Questa semantica è la proposta da provare. La sostituzione della lettera già ricevuta dal destinatario è il punto tecnico più delicato e deve superare le verifiche di compatibilità prima di diventare il comportamento definitivo.

Il [contratto del primo prototipo](piano-ricreazione-rust.md#contratto-proposto-per-il-primo-gesto) precisa timer dal down del trigger, false partenze, ripetizioni e coppie down/up. La pressione che attiva non naviga; per selezionare con Spazio occorre rilasciarlo e premerlo di nuovo. Durante l'attesa, un rilascio anticipato della base restituisce una sola pressione di Spazio soltanto se il destinatario è ancora valido. Una volta aperto il selettore, il rilascio senza scelta conserva soltanto la base. Queste regole sono proposte da valutare nelle prove di scrittura.

## Annullamento e interruzioni

| Evento | Comportamento richiesto |
| --- | --- |
| Esc | Chiude il selettore e conserva la lettera base |
| Rilascio della lettera senza selezione | Conserva la lettera base |
| Cambio di finestra o perdita del contesto | Annulla la sessione e non invia la sostituzione alla nuova finestra |
| Altro carattere durante la selezione | Annulla e lascia proseguire la digitazione ordinaria |
| Applicazione esclusa o Accento in pausa | Non avvia una sessione di selezione |
| Errore di inserimento | Interrompe senza ripetizioni automatiche o cancellazioni compensative alla cieca |

Un errore parziale di inserimento può aver già modificato il testo. Il prototipo deve rilevare quanto possibile l'esito e documentare il limite, senza promettere una transazione atomica fra applicazioni.

## Regole dei caratteri

- L'ordine iniziale è stabile. La frequenza d'uso non modifica silenziosamente la disposizione.
- Shift e Caps Lock determinano la famiglia maiuscola o minuscola all'inizio della sessione, da confermare con prove sui layout supportati.
- Ctrl, Alt, AltGr e Windows impediscono l'attivazione. Una loro pressione durante la selezione annulla la sessione senza consumare la scorciatoia.
- Le lettere senza varianti non aprono un selettore vuoto.
- Per la prima versione si preferiscono caratteri precomposti dove disponibili. Sequenze combinanti e grafemi complessi richiedono una specifica dedicata.
- L'input prodotto da Accento deve essere riconosciuto per evitare riattivazioni ricorsive.

## Posizione e impostazioni

La prima posizione proposta è in basso al centro dell'area utile del monitor della finestra attiva. La barra deve rimanere dentro l'area utile e adattarsi alla scala del monitor. Il rilevamento del cursore di testo arriverà come modalità aggiuntiva con ritorno alla posizione fissa.

La finestra impostazioni riceve il focus quando viene aperta esplicitamente. Un campo di prova mostra come usare il gesto e consente di sperimentare il ritardo. L'area di notifica offre almeno Pausa, Riprendi, Impostazioni ed Esci.

## Contesti da trattare separatamente

IME, tasti morti, terminali, sessioni remote, giochi e applicazioni elevate richiedono prove dedicate. Per composizioni o layout non supportati si preferisce non attivare il selettore. Il riconoscimento affidabile di questi contesti deve essere verificato.

La disattivazione nei campi password è un obiettivo da esplorare: un hook globale non garantisce di riconoscere tutti i campi protetti. Questa capacità non va dichiarata finché non è dimostrata per i contesti supportati.

