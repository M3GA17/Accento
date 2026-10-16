import {readFile,writeFile,mkdir} from 'node:fs/promises';
const languages=JSON.parse(await readFile('localization/languages.json','utf8'));
const characters=JSON.parse(await readFile('localization/characters.json','utf8'));
const currencies=JSON.parse(await readFile('localization/currencies.json','utf8'));
const messages=JSON.parse(await readFile('localization/messages.json','utf8'));
await mkdir('docs/locales',{recursive:true});
const rows=[];
for(const language of languages){
  const t=messages[language.code];
  const maps=characters[language.code].maps;
  let guide=`# ${t.GuideTitle} — ${language.name}\n\n> ${t.GuideReview}\n\n`;
  for(const key of ['GuideStart','GuideWrite','GuideSets','GuideLanguage','GuideSave','GuideExit','GuideWindows','GuideAlphabet'])guide+=(key==='GuideSets'?`${t.General} → ${t.Sets}\n\n`:'')+t[key]+'\n\n';
  guide+=`## ${t.Sets} · ${language.name}\n\n${language.alphabet}\n\n`;
  guide+=`| ${t.Writing} | ${t.Variants} |\n| --- | --- |\n`;
  for(const [base,variants] of Object.entries(maps)){
    if(!/^[a-z']$/.test(base))continue;
    guide+=`| \`${base}\` | ${[...new Set(variants.flat())].map(value=>'`'+value+'`').join(' · ')} |\n`;
  }
  guide+=`\n## ${t.SpecialSets} · ${t.Currency}\n\n${t.General} → ${t.Sets} → ${t.SpecialSets} → ${t.Currency}\n\n| ${t.Writing} | ${t.Variants} |\n| --- | --- |\n`;
  for(const [base,symbols] of Object.entries(currencies))guide+=`| \`${base}\` | ${symbols.map(symbol=>'`'+symbol+'`').join(' · ')} |\n`;
  guide+='\n[Unicode CLDR 48](https://github.com/unicode-org/cldr-json/tree/48.0.0) · [Coverage / Fonti](../copertura-europa.md)\n';
  await writeFile(`docs/locales/${language.code}.md`,guide);
  rows.push(`| ${language.code} | ${language.name} | ${language.countries.join(', ')} | ${language.script} | [Guida](locales/${language.code}.md) |`);
}
const coverage=`# Copertura europea di Accento\n\nIl perimetro concordato comprende le lingue scritte ufficiali o nazionali degli Stati europei, microstati, Stati transcontinentali e Caucaso. Sono disponibili **${languages.length} voci**, contando separatamente Bokmål/Nynorsk e le varianti latine/cirilliche di bosniaco, serbo e montenegrino. Il latino è un supplemento istituzionale per la Santa Sede. Non è una promessa di copertura di ogni lingua regionale o minoritaria europea. [Fonti e precisazioni di status](copertura-europa-fonti.md).\n\nOgni voce è presente in interfaccia, catalogo e guida rapida. Le nuove traduzioni sono bozze automatiche o redatte dall'assistente, con correzioni terminologiche esplicite; **non sono state revisionate da madrelingua**. Provenienza e stato sono registrati in localization/translation-status.json. Le guide localizzate coprono l'uso corrente; la ricerca e i rapporti storici restano nella lingua originale.\n\n| Codice | Lingua / variante | Paesi (codici geografici) | Scrittura | Documentazione |\n| --- | --- | --- | --- | --- |\n${rows.join('\n')}\n\n## Repertori e associazioni\n\nI repertori principali provengono da Unicode CLDR 48, fissato e incluso nel repository. Montenegrino e latino hanno integrazioni dichiarate. Le varianti russe con accento di tonicità sono un'estensione dal repertorio ausiliario CLDR. L'italiano conserva anche ó per compatibilità. Il motore gestisce candidati testuali completi, inclusi digrammi e sequenze con segni combinanti; non tronca una maiuscola a un singolo scalare.\n\nI tasti latini offrono associazioni pratiche per accedere anche agli altri alfabeti; non costituiscono un metodo di traslitterazione normativo. I layout nazionali riconosciuti usano la lettera tradotta da Windows, conservata fino al rilascio del tasto. I nomi dei tasti virtuali ASCII non sono più usati come identità della lettera.\n\n## Albanese\n\nL'alfabeto standard ha 36 lettere, inclusi i nove digrammi dh, gj, ll, nj, rr, sh, th, xh, zh. I diacritici sono ç/Ç ed ë/Ë: il precedente set era completo per i diacritici, ma non mostrava l'alfabeto intero. Ora la guida elenca tutte le lettere e il popup offre anche i digrammi, che rimangono digitabili normalmente.\n\n## Limiti delle verifiche\n\nI test controllano la raggiungibilità di ogni elemento non ordinario del repertorio principale, le maiuscole, i digrammi, i segni combinanti, la corrispondenza dei codici e la presenza di tutti i testi e delle guide. Questo non certifica tutte le applicazioni destinatarie, tutti i layout fisici, la qualità madrelingua o tutti i repertori storici/fonetici. I candidati lunghi e le molte varianti vengono navigati dal popup esistente, con la selezione mantenuta nel tratto visibile.\n`;
await writeFile('docs/copertura-europa.md',coverage);
console.log(`${languages.length} localized quick-start guides and coverage matrix generated.`);
