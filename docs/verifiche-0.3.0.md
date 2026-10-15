# Verifiche di Accento 0.3.0

Revisione del 15 ottobre 2026. La UI riprende il FigCraft 0.3: popup con soli tasti indipendenti 64 × 64, gap 8, altezza 80 e larghezza 72n + 8; impostazioni nelle quattro sezioni con navigazione laterale o tab compatti. Il motore degli accenti resta invariato.

## Materiale e composizione

Il popup usa `WS_EX_LAYERED`, una superficie BGRA premoltiplicata disegnata con Direct2D e `UpdateLayeredWindow`. La radice non riceve un backdrop DWM. Il materiale campiona soltanto il rettangolo del popup, prima della sua apertura, applica sfocatura e rifrazione ai singoli tasti, poi aggiunge tinta, bordi e glifi. Il campione resta in memoria per il gesto e viene rilasciato alla chiusura; non viene registrato durante l'uso normale. In modalità solida non viene campionato.

L'adattamento non replica il motore ottico Apple. Il campione non viene aggiornato continuamente mentre una sessione è aperta. L'alpha lascia trasparenti angoli e spazi fra tasti anche nella modalità solida; le ombre corte appartengono ai singoli candidati. L'assenza di label visive non rimuove il nome accessibile e gli annunci della selezione.

## Controlli eseguiti

Ambiente: Windows 11 x64 build 26300, Rust MSVC 1.98.0, windows 0.62.2, runtime C statico.

| Verifica | Esito |
| --- | --- |
| Test Rust | 21 superati: 14 motore, 2 preferenze, 2 layout, 3 materiale |
| Formattazione | Superata |
| Clippy su tutti i target, warning come errori | Superato |
| Build release | Superata |
| Collaudo Windows con hook e controlli reali | 35 controlli superati |
| Alpha della superficie Direct2D effettiva | Angoli 0; gap trasparenti in tutti e tre gli aspetti |

I test del materiale verificano alpha premoltiplicato, gap, geometria a DPI differenti, dipendenza del glass dallo sfondo, indipendenza della superficie solida e arrotondamenti con scala del testo frazionaria. I test del layout verificano che le dimensioni dei controlli non vengano ridotte alla larghezza e che il contenuto possa scorrere mantenendo il footer.

Il collaudo Windows comprende lingua multipla e ultima lingua conservata, temi esclusivi, interruttore Glass, stepper, errore inline, bozza conservata fra sezioni, ritorno al campo errato, focus che scorre nel viewport, Tab nel contenitore dei controlli, visibilità delle quattro sezioni compatte e stato disattivato di Salva senza modifiche. Seguono inserimento, maiuscole, Esc, falsa partenza, assenza di scelta, navigazione, cambio campo, pausa, esclusioni e i tre materiali.

Le prove di salvataggio della UI usano dati non validi e si arrestano prima della scrittura. I test Rust di persistenza scrivono in una cartella temporanea. Le preferenze reali dell'utente non vengono sostituite dal collaudo.

Le catture delle impostazioni, il popup sopra la finestra di prova posseduta da Accento, le superfici con alpha, il rapporto originale e l'hash dell'eseguibile sono in [evidenze/0.3.0](evidenze/0.3.0). Non sono render del FigCraft. Nelle catture delle impostazioni viene escluso il bordo invisibile DWM; la UI non viene ritoccata.

## Limiti ancora aperti

La verifica automatica dell'inserimento riguarda controlli Edit di prova. Word, browser, terminali, Narrator, contrasto elevato reale e la matrice multi-monitor/DPI richiedono prove dedicate. I test puri coprono la geometria e i pixel a scale diverse, non certificano una prova fisica multi-monitor. La build minima 22621, firma e installer non sono verificati qui.

Il progetto è stato inizializzato come repository Git locale, con uno snapshot di base precedente all'implementazione per rendere il commit della nuova UI confrontabile. Nessun repository remoto è stato creato e nessun push è stato eseguito.
