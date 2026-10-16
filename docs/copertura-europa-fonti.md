# Copertura linguistica europea: perimetro e fonti

Verifica delle fonti: 16 ottobre 2026. Questo documento stabilisce criteri di copertura; non certifica automaticamente che tutte le funzionalità qui descritte siano già implementate. Per i repertori si usa come riferimento riproducibile **Unicode CLDR release-48**, senza attribuirle il significato di ultima versione disponibile.

## Albanese: che cosa significa «completo»

Lo standard albanese ha 36 lettere: `a b c ç d dh e ë f g gj h i j k l ll m n nj o p q r rr s sh t th u v x xh y z zh`. Nove sono digrammi: `dh gj ll nj rr sh th xh zh`. Il testo normativo *Drejtshkrimi i gjuhës shqipe* (1973), sezione iniziale «Alfabeti i gjuhës shqipe», riporta il numero e l'alfabeto; la copia digitalizzata è ospitata da Shkenca. [Testo normativo, pagina PDF 3](https://www.shkenca.org/pdf/gjuhe/drejtshkrimi_i_gjuhes_shqipe_1973.pdf).

CLDR conferma questo repertorio nel set principale di `sq`, rappresentando esplicitamente i digrammi come stringhe. Nel set ausiliario figura `w`. Le sole lettere con diacritico dello standard sono **ç/Ç ed ë/Ë**. Per un selettore di varianti che integra la normale tastiera latina, `c → ç` ed `e → ë`, con le corrispondenti maiuscole, coprono quindi i diacritici standard. I digrammi si digitano normalmente come due lettere; non sono nove caratteri Unicode mancanti. Questo non promette copertura di trascrizioni dialettali, storiche o fonetiche. [Dati primari CLDR sq, elemento characters](https://github.com/unicode-org/cldr/blob/release-48/common/main/sq.xml).

## Perimetro europeo concordato

L'utente ha scelto **solo lingue ufficiali o nazionali degli Stati europei**, includendo microstati, Stati transcontinentali e Caucaso. Sono escluse le lingue riconosciute esclusivamente a livello regionale o minoritario. «Tutta l'Europa» indica quindi il perimetro qui elencato, non ogni lingua o varietà parlata sul continente; le sigle identificano lingue o varianti, non nazionalità.

| Gruppo | Lingue / identificatori da considerare |
| --- | --- |
| Le 24 lingue ufficiali dell'Unione europea | Bulgaro `bg`, ceco `cs`, croato `hr`, danese `da`, estone `et`, finlandese `fi`, francese `fr`, greco `el`, inglese `en`, irlandese `ga`, italiano `it`, lettone `lv`, lituano `lt`, maltese `mt`, neerlandese `nl`, polacco `pl`, portoghese `pt`, rumeno `ro`, slovacco `sk`, sloveno `sl`, spagnolo `es`, svedese `sv`, tedesco `de`, ungherese `hu` |
| Estensione europea, inclusi microstati e Caucaso | Albanese `sq`, armeno `hy`, azero latino `az`, bielorusso `be`, bosniaco latino `bs` e cirillico `bs-Cyrl`, catalano `ca`, georgiano `ka`, islandese `is`, kazako cirillico `kk`, latino `la`, lussemburghese `lb`, macedone `mk`, montenegrino latino `cnr` e cirillico `cnr-Cyrl`, norvegese Bokmål `nb` e Nynorsk `nn`, romancio `rm`, russo `ru`, serbo cirillico `sr` e latino `sr-Latn`, turco `tr`, ucraino `uk` |

Il primo gruppo deriva dalla [lista istituzionale UE](https://european-union.europa.eu/principles-countries-history/languages_en). L'estensione è una **scelta esplicita di ambito di Accento**: l'ufficialità a livello regionale non comporta inclusione automatica. Basco, gallese, galiziano, sami e altre lingue regionali restano fuori da questa implementazione. Catalano e romancio restano inclusi per il loro status nazionale, non per un ampliamento alle regionali.

Precisazioni sui casi meno evidenti:

- Il catalano è lingua ufficiale dello Stato di Andorra: [Costituzione, articolo 2](https://www.consellgeneral.ad/fitxers/documents/constitucio/const-cat).
- Il romancio è una delle quattro lingue nazionali svizzere, con uso ufficiale federale nelle comunicazioni con i romanciofoni: [Confederazione svizzera](https://www.eda.admin.ch/en/four-languages-four-cultures-one-country), [Legge federale sulle lingue](https://www.fedlex.admin.ch/eli/cc/2009/821/en).
- Il lussemburghese è lingua nazionale e amministrativa: [Governo lussemburghese](https://gouvernement.lu/en/dossiers/2018/langue-luxembourgeoise.html).
- Bokmål e Nynorsk sono due standard scritti norvegesi di pari valore: [Språklova, sezione 4](https://lovdata.no/dokument/NLE/lov/2021-05-21-42). Non usare `no` come sostituto indistinto dei due cataloghi.
- Il latino è incluso per la tradizione istituzionale della Santa Sede; evitare di descriverlo semplicisticamente come lingua ufficiale dello Stato della Città del Vaticano. [Motu proprio Latina Lingua](https://www.vatican.va/content/benedict-xvi/it/motu_proprio/documents/hf_ben-xvi_motu-proprio_20121110_latina-lingua.html).
- Kazako, azero, armeno e georgiano rendono esplicita l'inclusione dei Paesi transcontinentali e del Caucaso: non sono aggiunte regionali. Il kazako è lingua di Stato e il russo ha uso ufficiale nelle istituzioni kazake: [Costituzione sul sito della Presidenza, articolo 9 nella versione consultata](https://www.akorda.kz/ru/konstituciya-respubliki-kazahstan-3053339).

Lista operativa unica, **47 voci incluse varianti scritte**: `it en sq fr es de pt ro pl tr nl lb rm da sv nb nn fi is et lv lt cs sk hu sl hr bs sr sr-Latn cnr mk bg el be uk ru ga mt ca hy ka az kk la`. Questo è il perimetro da confrontare fra interfaccia, catalogo dei set e guida; non si aggiungono lingue delle singole repubbliche federate o territori autonomi sulla sola base dell'ufficialità locale. Il criterio comprende l'uso nazionale di fatto dell'inglese, senza attribuirgli una specifica norma costituzionale britannica.

Controllo di completezza geografica della lista: i microstati non richiedono «monegasco» o «liechtensteinese»: sono già coperti dal francese per Monaco, dal tedesco per Liechtenstein, dall'italiano per San Marino e dal catalano per Andorra. Svizzera e Lussemburgo richiedono invece `rm` e `lb`; Kosovo usa lingue già incluse (`sq`, `sr`); Moldavia usa `ro`; Cipro usa `el` e `tr`; Bosnia-Erzegovina è rappresentata da `bs`, `hr` e `sr`. Le due varianti norvegesi, la variante serba latina e il latino spiegano perché il numero di voci non coincide con un numero di Stati o di lingue distinte.

## Montenegrino e latino

Il montenegrino richiede un repertorio esplicito `cnr`, senza etichettare un set serbo come montenegrino. Il *Pravopis crnogorskoga jezika i rječnik crnogorskoga jezika*, pubblicazione del Ministero dell'Istruzione, include **ś/Ś e ź/Ź**, oltre ai caratteri latini condivisi con le lingue vicine. [Testo normativo, copia del Consiglio della minoranza montenegrina di Zagabria](https://www.vijece-crnogoraca-zagreb.hr/media/9504/pravopis-crnogorskog-jezika-rjecnik.pdf). La norma è richiamata anche da una [decisione del Difensore civico montenegrino](https://www.ombudsman.co.me/docs/1606300830_28102020-misljenje-pg.pdf). Le corrispondenti forme cirilliche con accento, `с́` e `з́`, sono sequenze di più scalari: documentarle come escluse finché il motore non supporta candidati stringa. Non dedurre la copertura cirillica completa dalla disponibilità del solo set latino.

Per il latino il set può offrire le vocali lunghe **ā ē ī ō ū ȳ** e maiuscole, dichiarandole segni didattici/editoriali della quantità vocalica. Non sono lettere aggiuntive obbligatorie in ogni testo latino. Eventuali brevi `ă ĕ ĭ ŏ ŭ` e legature devono essere ampliamenti dichiarati, non requisiti impliciti della traduzione latina. Esempi primari nella grammatica pubblicata da Dickinson mostrano l'uso distinto di macron e breve: [Allen and Greenough, Accents](https://dcc.dickinson.edu/ro/grammar/latin/accents).

## Repertori e limiti tecnici

CLDR distingue set principale, ausiliario, indice, numeri e punteggiatura. Per Accento conviene partire dalle lettere principali e dichiarare separatamente gli ampliamenti ausiliari. Un exemplar set è un repertorio di dati linguistici, non una mappa di tasti né un pacchetto di traduzioni dell'app. Non basta eliminare le parentesi e iterare sui caratteri: vi sono intervalli, escape e stringhe. [Specifica LDML, Character Elements](https://unicode.org/reports/tr35/tr35-general.html#Character_Elements).

| Scrittura | Implicazione per Accento |
| --- | --- |
| Latina | Estrarre lettere distintive; validare manualmente tasto base e maiuscole, soprattutto `i/ı/İ/I`, `ß/ẞ`, legature e digrammi. Le lettere ASCII normali non devono riempire inutilmente i popup. |
| Greca | Coprire il greco moderno monotonico, inclusi tonos e dieresi; dichiarare separatamente il politonico ausiliario. Non confondere lettere greche e omografi latini. [CLDR el](https://github.com/unicode-org/cldr/blob/release-48/common/main/el.xml). |
| Cirillica | Mantenere repertori distinti per lingua. Il serbo include `ђ ј љ њ ћ џ`; non basta un set russo comune. La variante latina deve restare distinta. [CLDR sr](https://github.com/unicode-org/cldr/blob/release-48/common/main/sr.xml). |
| Armena | Definire una mappa di inserimento e verificare la resa delle lettere; `և` compare nel set ausiliario CLDR. Un'etichetta tradotta non costituisce supporto della tastiera. [CLDR hy](https://github.com/unicode-org/cldr/blob/release-48/common/main/hy.xml). |
| Georgiana | Il repertorio moderno principale e quello ausiliario sono differenti; non abilitare indiscriminatamente tutte le forme storiche. [CLDR ka](https://github.com/unicode-org/cldr/blob/release-48/common/main/ka.xml). |

Il catalogo originario di Accento usa candidati `Vec<char>`: ogni candidato è un singolo valore scalare Unicode. Ciò esclude sequenze di più scalari come la vocale russa con accento di tonicità `а́` (U+0430 + U+0301), presente nel set ausiliario CLDR russo. Per supportarle servono candidati stringa, selezione/inserimento coerenti e prove sulla cancellazione. Non dichiarare la tonicità russa coperta solo perché sono presenti le 33 lettere di base. [CLDR ru](https://github.com/unicode-org/cldr/blob/release-48/common/main/ru.xml).

Un carattere percepito dall'utente può comprendere più scalari; la segmentazione in grafemi è definita da [Unicode UAX #29](https://www.unicode.org/reports/tr29/). Normalizzare in NFC aiuta dove esiste una forma precomposta, ma non trasforma tutte le sequenze in un singolo scalare. [Unicode UAX #15](https://www.unicode.org/reports/tr15/).

## Criteri di accettazione e documentazione

Questi sono criteri di progetto, non affermazioni ricavate automaticamente dalle fonti:

1. Tenere una matrice con tre stati indipendenti per ogni lingua: interfaccia, set di inserimento, guida utente. Un fallback inglese deve essere dichiarato come fallback, non come traduzione completa.
2. Per l'interfaccia verificare tutte le chiavi, segnaposto, testi lunghi, nomi nativi e maiuscole. Le traduzioni iniziali richiedono revisione di un parlante competente prima di essere definite revisionate.
3. Per i set registrare fonte e versione, principali e ausiliari inclusi, mappa dei tasti, maiuscole, eventuali stringhe non supportate e casi esclusi. Provare almeno un carattere distintivo di ciascun set.
4. La guida in ogni lingua deve spiegare attivazione dei set, gesto di apertura, selezione, annullamento e limiti. Manuale tradotto e dati CLDR hanno scopi diversi.
5. Per scritture non latine precisare se il prodotto offre varianti della tastiera corrente o una mappa di translitterazione latina: un popup di accenti non sostituisce automaticamente un layout completo o un IME.

## Esito dell’implementazione 0.6

La matrice definitiva comprende 47 voci, aggiungendo le varianti cirilliche di bosniaco e montenegrino. Il precedente limite `Vec<char>` è stato rimosso: il motore usa candidati `String` completi, conserva maiuscole composte, digrammi e segni combinanti, inclusi `с́`, `з́`, `а́` e la forma olandese `íj́`. Il repertorio principale CLDR viene verificato automaticamente; il repertorio ausiliario non è incorporato integralmente. La tonicità russa e i macron latini sono estensioni esplicite. Stato delle traduzioni, registro e guide sono descritti in [copertura-europa.md](copertura-europa.md).
