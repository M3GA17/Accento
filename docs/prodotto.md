# Prodotto e perimetro di Accento

Accento aiuta a digitare caratteri che non sono immediatamente disponibili sulla tastiera. L'obiettivo è completare una parola senza aprire una mappa caratteri, copiare testo o interrompere il lavoro nell'applicazione corrente.

## Destinatari e bisogni

Il pubblico iniziale comprende chi usa tastiere con layout internazionale, scrive in più lingue o cerca un accesso semplice alle maiuscole accentate. Esempi rappresentativi sono scrivere `È corretto`, scegliere tra `è` ed `é` e inserire `ñ` senza cambiare layout.

Il valore del prodotto dipende da prevedibilità, facilità di apprendimento e affidabilità dell'inserimento. L'uso di Rust e il materiale Glass sostengono questi obiettivi; vantaggi di prestazioni o usabilità richiedono misurazioni comparative.

## Decisioni acquisite

- Il nome del progetto è Accento.
- L'app deve funzionare indipendentemente da PowerToys.
- Il motore deve essere scritto in Rust.
- La direzione visiva deve ispirarsi al linguaggio Apple Liquid Glass.
- La definizione iniziale precede il prototipo; lo stato aggiornato dell'implementazione è nel [rapporto 0.1.0](verifiche-0.1.0.md).

## Perimetro proposto della prima versione

Windows 11 è la piattaforma iniziale proposta. La versione minima e le architetture supportate saranno fissate dopo le prove delle API e del rendering.

| Priorità | Capacità | Esito atteso |
| --- | --- | --- |
| Essenziale | Attivazione con lettera e tasto aggiuntivo | Selezione intenzionale senza alterare la digitazione ordinaria |
| Essenziale | Selezione da tastiera e annullamento | Uso completo senza mouse |
| Essenziale | Set linguistici e maiuscole | Candidati pertinenti e coerenti con il carattere di partenza |
| Essenziale | Area di notifica con pausa e uscita | Controllo immediato sul comportamento dell'app |
| Essenziale | Impostazioni locali e campo di prova | Configurazione comprensibile e verificabile |
| Essenziale | Elenco delle applicazioni escluse | Possibilità di evitare interferenze nei programmi scelti |
| Essenziale | Aspetto chiaro, scuro e opaco | Leggibilità su sfondi diversi |
| Successiva | Attivazione con sola pressione prolungata | Alternativa per chi preferisce quel gesto |
| Successiva | Posizione vicino al cursore di testo | Minore distanza visiva quando il cursore è individuabile |
| Successiva | Ordine personalizzato e preferiti | Adattamento alle abitudini individuali |

Il primo set di prova comprende italiano; francese, spagnolo e tedesco ampliano la verifica prima della prima versione pubblica. I set si combinano eliminando i duplicati. Non si dichiara copertura linguistica completa senza una revisione delle relative mappature.

## Funzioni rinviate

La prima versione non comprende sincronizzazione cloud, account, correzione automatica, riscrittura del testo, macro, espansione di frasi o un catalogo generale di emoji. macOS e Linux richiedono integrazioni proprie e non fanno parte della prima consegna.

## Fiducia e distribuzione

La configurazione rimane sul dispositivo. Accento non deve registrare frasi, cronologia dei tasti o contenuti dei campi; anche i log diagnostici devono rispettare questa regola. Eventuali statistiche aggregate sui caratteri saranno una decisione successiva e facoltativa.

Installer, versione portabile, firma e canale di aggiornamento restano da decidere. Il nome Accento è scelto per il progetto; questa scelta non equivale a una verifica della disponibilità commerciale di marchio, dominio o nome negli store.

