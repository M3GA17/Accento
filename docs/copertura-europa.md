# Copertura europea di Accento

Il perimetro concordato comprende le lingue scritte ufficiali o nazionali degli Stati europei, microstati, Stati transcontinentali e Caucaso. Sono disponibili **47 voci**, contando separatamente Bokmål/Nynorsk e le varianti latine/cirilliche di bosniaco, serbo e montenegrino. Il latino è un supplemento istituzionale per la Santa Sede. Non è una promessa di copertura di ogni lingua regionale o minoritaria europea. [Fonti e precisazioni di status](copertura-europa-fonti.md).

Ogni voce è presente in interfaccia, catalogo e guida rapida. Le nuove traduzioni sono bozze automatiche o redatte dall'assistente, con correzioni terminologiche esplicite; **non sono state revisionate da madrelingua**. Provenienza e stato sono registrati in localization/translation-status.json. Le guide localizzate coprono l'uso corrente; la ricerca e i rapporti storici restano nella lingua originale.

| Codice | Lingua / variante | Paesi (codici geografici) | Scrittura | Documentazione |
| --- | --- | --- | --- | --- |
| it | Italiano | IT, SM, VA | Latin | [Guida](locales/it.md) |
| en | English | GB, IE, MT | Latin | [Guida](locales/en.md) |
| sq | Shqip | AL, XK | Latin | [Guida](locales/sq.md) |
| fr | Français | FR, BE, CH, LU, MC | Latin | [Guida](locales/fr.md) |
| es | Español | ES | Latin | [Guida](locales/es.md) |
| de | Deutsch | DE, AT, BE, CH, LI, LU | Latin | [Guida](locales/de.md) |
| pt | Português | PT | Latin | [Guida](locales/pt.md) |
| ro | Română | RO, MD | Latin | [Guida](locales/ro.md) |
| pl | Polski | PL | Latin | [Guida](locales/pl.md) |
| tr | Türkçe | TR, CY | Latin | [Guida](locales/tr.md) |
| nl | Nederlands | NL, BE | Latin | [Guida](locales/nl.md) |
| lb | Lëtzebuergesch | LU | Latin | [Guida](locales/lb.md) |
| rm | Rumantsch | CH | Latin | [Guida](locales/rm.md) |
| da | Dansk | DK | Latin | [Guida](locales/da.md) |
| sv | Svenska | SE, FI | Latin | [Guida](locales/sv.md) |
| nb | Norsk bokmål | NO | Latin | [Guida](locales/nb.md) |
| nn | Norsk nynorsk | NO | Latin | [Guida](locales/nn.md) |
| fi | Suomi | FI | Latin | [Guida](locales/fi.md) |
| is | Íslenska | IS | Latin | [Guida](locales/is.md) |
| et | Eesti | EE | Latin | [Guida](locales/et.md) |
| lv | Latviešu | LV | Latin | [Guida](locales/lv.md) |
| lt | Lietuvių | LT | Latin | [Guida](locales/lt.md) |
| cs | Čeština | CZ | Latin | [Guida](locales/cs.md) |
| sk | Slovenčina | SK | Latin | [Guida](locales/sk.md) |
| hu | Magyar | HU | Latin | [Guida](locales/hu.md) |
| sl | Slovenščina | SI | Latin | [Guida](locales/sl.md) |
| hr | Hrvatski | HR, BA | Latin | [Guida](locales/hr.md) |
| bs | Bosanski | BA | Latin | [Guida](locales/bs.md) |
| bs-Cyrl | Босански (ћирилица) | BA | Cyrillic | [Guida](locales/bs-Cyrl.md) |
| sr | Српски | RS, BA, XK | Cyrillic | [Guida](locales/sr.md) |
| sr-Latn | Srpski (latinica) | RS, BA, XK | Latin | [Guida](locales/sr-Latn.md) |
| cnr | Crnogorski | ME | Latin | [Guida](locales/cnr.md) |
| cnr-Cyrl | Црногорски (ћирилица) | ME | Cyrillic | [Guida](locales/cnr-Cyrl.md) |
| mk | Македонски | MK | Cyrillic | [Guida](locales/mk.md) |
| bg | Български | BG | Cyrillic | [Guida](locales/bg.md) |
| el | Ελληνικά | GR, CY | Greek | [Guida](locales/el.md) |
| be | Беларуская | BY | Cyrillic | [Guida](locales/be.md) |
| uk | Українська | UA | Cyrillic | [Guida](locales/uk.md) |
| ru | Русский | RU, BY, KZ | Cyrillic | [Guida](locales/ru.md) |
| ga | Gaeilge | IE | Latin | [Guida](locales/ga.md) |
| mt | Malti | MT | Latin | [Guida](locales/mt.md) |
| ca | Català | AD | Latin | [Guida](locales/ca.md) |
| hy | Հայերեն | AM | Armenian | [Guida](locales/hy.md) |
| ka | ქართული | GE | Georgian | [Guida](locales/ka.md) |
| az | Azərbaycanca | AZ | Latin | [Guida](locales/az.md) |
| kk | Қазақша | KZ | Cyrillic | [Guida](locales/kk.md) |
| la | Latina | VA | Latin | [Guida](locales/la.md) |

## Repertori e associazioni

I repertori principali provengono da Unicode CLDR 48, fissato e incluso nel repository. Montenegrino e latino hanno integrazioni dichiarate. Le varianti russe con accento di tonicità sono un'estensione dal repertorio ausiliario CLDR. L'italiano conserva anche ó per compatibilità. Il motore gestisce candidati testuali completi, inclusi digrammi e sequenze con segni combinanti; non tronca una maiuscola a un singolo scalare.

I tasti latini offrono associazioni pratiche per accedere anche agli altri alfabeti; non costituiscono un metodo di traslitterazione normativo. I layout nazionali riconosciuti usano la lettera tradotta da Windows, conservata fino al rilascio del tasto. I nomi dei tasti virtuali ASCII non sono più usati come identità della lettera.

## Albanese

L'alfabeto standard ha 36 lettere, inclusi i nove digrammi dh, gj, ll, nj, rr, sh, th, xh, zh. I diacritici sono ç/Ç ed ë/Ë: il precedente set era completo per i diacritici, ma non mostrava l'alfabeto intero. Ora la guida elenca tutte le lettere e il popup offre anche i digrammi, che rimangono digitabili normalmente.

## Limiti delle verifiche

I test controllano la raggiungibilità di ogni elemento non ordinario del repertorio principale, le maiuscole, i digrammi, i segni combinanti, la corrispondenza dei codici e la presenza di tutti i testi e delle guide. Questo non certifica tutte le applicazioni destinatarie, tutti i layout fisici, la qualità madrelingua o tutti i repertori storici/fonetici. I candidati lunghi e le molte varianti vengono navigati dal popup esistente, con la selezione mantenuta nel tratto visibile.
