import {readFile,writeFile} from 'node:fs/promises';
const source = await readFile('crates/accento-platform/src/i18n.rs','utf8');
const languages = JSON.parse(await readFile('localization/languages.json','utf8'));
const codes = languages.map(language=>language.code);
const baseCodes = ['it','fr','es','de','en','sq'];
const messages = JSON.parse(await readFile('localization/messages.json','utf8'));
for(const [,key,values] of source.matchAll(/^\s+(\w+) => (\["[^\n]+\]),$/gm)) {
  const translations = JSON.parse(values);
  if(translations.length !== baseCodes.length) throw new Error(`Incomplete base translations: ${key}`);
  baseCodes.forEach((code,i) => {
    if(messages[code]?.[key]!==translations[i]) throw new Error(`Dictionary out of sync: ${code}/${key}`);
  });
}
if(Object.keys(messages.it).length < 50) throw new Error('No translation dictionary found');
for(const code of codes){
  if(!messages[code])throw new Error(`Missing locale: ${code}`);
  for(const key of Object.keys(messages.en))if(!messages[code][key]?.trim())throw new Error(`Missing translation: ${code}/${key}`);
}
if(Object.keys(messages).length!==codes.length)throw new Error('Locale registry and dictionaries differ');
const uiMessages=Object.fromEntries(Object.entries(messages).map(([code,dictionary])=>[code,Object.fromEntries(Object.entries(dictionary).filter(([key])=>!key.startsWith('Guide')))]));
await writeFile('ui/src/messages.json',JSON.stringify(uiMessages,null,2)+'\n');
