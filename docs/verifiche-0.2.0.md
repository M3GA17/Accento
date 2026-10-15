# Verifiche di Accento 0.2.0

Il rework del 15 ottobre 2026 sostituisce la precedente grafica con tessere quadrate indipendenti per candidati, lingue e aspetto. Il motore degli accenti mantiene il contratto della versione precedente.

## Interfaccia

Il selettore usa candidati 64 × 64 DIP con selezione mediante tinta, bordo, carattere e punto. Le impostazioni hanno opzioni 144 × 144, miniature dei temi, monogramma, keycap, ritardo regolabile, campi in pannelli distinti e footer con salvataggio o errore. Il renderer condiviso disegna gradienti diagonali, due bordi ottici e ombre. Il selettore conserva Acrylic quando disponibile; le impostazioni usano un target opaco compatibile con i controlli Edit Windows.

I controlli rimangono finestre native raggiungibili con Tab; i nomi delle tessere includono lo stato selezionato. Hover, pressione, focus dei campi e degli altri controlli hanno segnali distinti. Il focus porta il comando in vista nella finestra ridotta. La verifica completa con Narrator e contrasto elevato delle nuove impostazioni rimane aperta.

## Collaudo

Ambiente: Windows 11 x64 build 26300, Rust 1.98.0 MSVC, windows 0.62.2. Compilazione release con runtime C statico.

| Verifica | Esito |
| --- | --- |
| Test Rust | 14 del motore e 2 delle preferenze superati |
| Clippy, tutti i target, warning come errori | Superato |
| Formattazione | Superata |
| Compilazione release | Superata |
| Collaudo Windows reale | 26 controlli superati |

Gli otto nuovi controlli verificano selezione della lingua, mantenimento dell'ultima lingua, temi esclusivi e anteprima, toggle Glass, stepper, rifiuto inline del ritardo non valido, scorrimento compatto e raggiungibilità di Salva. Gli altri 18 verificano il gesto e l'inserimento con hook reali, focus, maiuscole, annullamento, pausa, esclusione e tre aspetti del selettore.

Le variazioni del collaudo restano in memoria. La prova del salvataggio della UI usa un valore non valido, verificando il rifiuto prima della scrittura. Le prove di persistenza Rust usano una cartella temporanea. I test terminano con codice 0; il renderer DC è rilasciato prima della chiusura COM.

Rapporto originale, catture della sola interfaccia e hash sono in [evidenze/0.2.0](evidenze/0.2.0). Sono state osservate le impostazioni chiare/scure, la finestra compatta e il selettore. I risultati visivi riguardano la macchina e la scala del collaudo.

## Design system

Il progetto FigCraft **della revisione 0.2**, conservato nell'[archivio precedente al rework](../design/archive/figcraft-before-liquid-glass.zip), conteneva 8 pagine, 42 componenti principali, 8 set, 43 variabili e 27 stili tipografici. Era stato salvato, riaperto e ispezionato; tutte le 15 tavole erano state esportate. Il [design corrente 0.3](../design/figcraft/README.md) propone una UI successiva e non modifica le evidenze del runtime riportate qui.

Restano aperte la matrice DPI multi-monitor, Narrator, applicazioni esterne come Word/browser/terminali, prestazioni misurate, build minima 22621 e distribuzione firmata. Le tavole FigCraft non estendono automaticamente le evidenze di accessibilità o compatibilità del software.
