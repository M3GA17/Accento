import {execFileSync} from 'node:child_process';
import {copyFile,cp,mkdir,readFile,readdir,realpath,rm,writeFile} from 'node:fs/promises';
import path from 'node:path';
const platform=process.platform;
const architecture=process.arch==='arm64'?'aarch64':'x86_64';
const target=process.env.CARGO_BUILD_TARGET??`${architecture}-${platform==='darwin'?'apple-darwin':platform==='win32'?'pc-windows-msvc':'unknown-linux-gnu'}`;
const output=path.resolve('target/release-resources');
await mkdir('target',{recursive:true});
const root=await realpath('.'),targetRoot=await realpath('target');
if(!targetRoot.startsWith(root+path.sep))throw new Error('Resource staging must stay inside the workspace target directory.');
await rm(output,{recursive:true,force:true});
await mkdir(output,{recursive:true});
const files={
  [`docs/distribution/${platform==='win32'?'README.txt':platform==='darwin'?'README-macos.txt':'README-linux.txt'}`]:'README.txt',
  'docs/uso.md':'LEGGIMI.md','docs/cross-platform.md':'PLATFORMS.md','LICENSE':'LICENSE',
  'THIRD-PARTY-NOTICES.md':'THIRD-PARTY-NOTICES.md','localization/LICENSE-UNICODE.txt':'LICENSE-UNICODE.txt',
  'docs/copertura-europa-fonti.md':'copertura-europa-fonti.md',
};
for(const [source,name]of Object.entries(files))await copyFile(source,path.join(output,name));
if(platform==='darwin')await copyFile('docs/distribution/TEST-MACOS.it.txt',path.join(output,'TEST-MACOS.it.txt'));
await writeFile(path.join(output,'LEGGIMI.md'),(await readFile('docs/uso.md','utf8')).replaceAll('](cross-platform.md)','](PLATFORMS.md)'));
await cp('docs/locales',path.join(output,'guide'),{recursive:true});
await writeFile(path.join(output,'copertura-europa.md'),(await readFile('docs/copertura-europa.md','utf8')).replaceAll('](locales/','](guide/'));
execFileSync(process.execPath,['scripts/collect-ui-licenses.mjs',output],{stdio:'inherit'});
const metadata=JSON.parse(execFileSync('cargo',['metadata','--format-version','1','--locked','--filter-platform',target],{encoding:'utf8',maxBuffer:12*1024*1024}));
for(const entry of metadata.packages.filter(entry=>entry.source)){
  const source=path.dirname(entry.manifest_path),folder=path.join(output,'licenses',`${entry.name}-${entry.version}`);
  await mkdir(folder,{recursive:true});
  for(const file of await readdir(source,{withFileTypes:true}))if(file.isFile()&&/^(LICENSE|LICENCE|COPYRIGHT|NOTICE|UNLICENSE)/i.test(file.name))await copyFile(path.join(source,file.name),path.join(folder,file.name));
}
console.log(`Distribution resources prepared for ${target}; nothing published.`);
