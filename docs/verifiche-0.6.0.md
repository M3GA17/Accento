# Verifiche Accento 0.6.0

Data: 16 ottobre 2026. Ambiente: Windows 11 x64 build 26300, WebView2 155, toolchain Rust MSVC e dipendenze fissate nei lockfile.

## Risultato

- Build React / TypeScript e release Rust con risorse web incorporate: superate.
- **39 test Rust**: 22 del motore/catalogo e 17 di integrazione locale, preferenze, traduzioni, composizione input e rendering.
- **8 scenari UI Playwright**: scuro fisso con sistema chiaro/scuro; assenza di Aspetto e link ai set in Generale; cambio e ricerca delle 47 voci; indipendenza lingua/set; ricerca dei set; digrammi e scritture non latine; validazione, pausa e annullamento; BlackList; tastiera, formato compatto e audit WCAG A/AA senza violazioni serious/critical nei flussi verificati.
- **9 verifiche Tauri/WebView2 reali**, con andata/ritorno di tutte le **47 coppie lingua/set**. I salvataggi con tema chiaro vengono normalizzati a scuro. Controllati persistenza, rifiuto di configurazioni invalide, eventi, pausa, protezione della bozza e chiusura della webview senza terminare il backend. URL release: `http://tauri.localhost/`, senza server di sviluppo.
- `cargo fmt` e `cargo clippy --all-targets -- -D warnings`: superati.
- Regressione nativa storica `--ui-test`: superata, 14 controlli e zero ridisegni inutili nei dieci eventi a limite scorrimento. Non è la UI mostrata normalmente da Accento.
- Libreria: controllo tipi, contratti SSR, copertura dei token e suite della libreria superati. Il selettore Flat Product 0.1.1 è verificato con un solo contenitore di scorrimento raggiungibile da tastiera.

## Copertura dei dati

Il test del catalogo verifica che ogni elemento non ordinario del repertorio principale sia raggiungibile, compresi digrammi e sequenze con segni combinanti. Sono presenti test specifici per l'albanese `ç`, `ë`, `dh`, il montenegrino cirillico `с́`, il russo `а́`, l'olandese `íj́`, le forme greche maiuscole composte e la distinzione turca `I/İ/ı/i`. L'inserimento mantiene tutte le unità UTF-16 in un unico batch con una sola sostituzione della lettera originale.

Ogni voce ha un dizionario esplicito e una guida rapida generata. Le traduzioni nuove restano **bozze senza revisione madrelingua**; i test verificano completezza e collegamenti, non qualità linguistica. [Matrice e limiti](copertura-europa.md), [fonti](copertura-europa-fonti.md).

## Limiti delle prove

Il collaudo Tauri isolato disabilita gli hook e usa preferenze temporanee, preservando l'istanza personale in esecuzione. Il collaudo completo con digitazione sintetica `--self-test` non è stato rieseguito in questa sessione: un'altra istanza di Accento 0.5 era attiva. I test del motore, della traduzione dei tasti e del batch Unicode non certificano ogni layout fisico, ogni applicazione destinataria o gli IME. Narrator e i diversi layout nazionali richiedono ancora una verifica manuale dedicata.

Le catture correnti sono in `docs/evidenze/europe/`; il rapporto desktop macchina è in `docs/evidenze/react-tauri/desktop-test.json`. Le catture contrassegnate Anteprima usano dati temporanei; `desktop-albanian-dark.png` proviene da WebView2 reale.
