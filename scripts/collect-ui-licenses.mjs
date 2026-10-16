import {readFile,writeFile,mkdir,readdir,copyFile} from 'node:fs/promises';
import path from 'node:path';
const lock=JSON.parse(await readFile('package-lock.json','utf8'));
const destination=process.argv[2];
const rows=[];
for(const [location,entry] of Object.entries(lock.packages)){
  if(!location.startsWith('node_modules/') || entry.dev)continue;
  const metadata=JSON.parse(await readFile(path.join(location,'package.json'),'utf8'));
  rows.push({name:metadata.name,version:metadata.version,license:metadata.license??'Pacchetto locale DimaLabs; licenza non dichiarata nel manifest'});
  if(destination){
    const folder=path.join(destination,'licenses',metadata.name.replaceAll('/','-')+'-'+metadata.version);
    await mkdir(folder,{recursive:true});
    for(const file of await readdir(location))if(/^(licen[cs]e|notice|copyright|unlicense)/i.test(file)){
      try{await copyFile(path.join(location,file),path.join(folder,file));}catch(error){if(error.code!=='EISDIR')throw error;}
    }
  }
}
rows.sort((a,b)=>a.name.localeCompare(b.name));
const text='# Dipendenze dell’interfaccia React\n\nInventario dei pacchetti di produzione fissati in `package-lock.json`. La build conserva i testi delle licenze disponibili nei pacchetti nella cartella `licenses` del portabile.\n\n| Pacchetto | Versione | Licenza dichiarata |\n| --- | --- | --- |\n'+rows.map(row=>`| ${row.name} | ${row.version} | ${row.license} |`).join('\n')+'\n';
await writeFile('docs/dependencies-ui.md',text);
if(destination)await writeFile(path.join(destination,'DEPENDENCIES-UI.md'),text);
console.log(`${rows.length} pacchetti UI di produzione censiti.`);
