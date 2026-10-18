import {copyFile,cp,mkdir,readdir,readFile,writeFile} from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import path from 'node:path';
if(!['linux','darwin'].includes(process.platform))throw new Error('Unix packaging must run on Linux or macOS.');
const {version}=JSON.parse(await readFile('package.json','utf8'));
const args=process.argv.slice(2),targetIndex=args.indexOf('--target');
const target=targetIndex>=0?args[targetIndex+1]:process.env.CARGO_BUILD_TARGET;
const cargoTarget=path.resolve(process.env.CARGO_TARGET_DIR??'target');
const bundle=path.join(cargoTarget,target??'',args.includes('--debug')?'debug':'release','bundle');
const output=path.resolve('dist/releases',`v${version}`,process.platform==='darwin'?'macos':'linux');
await mkdir(output,{recursive:true});
const assets=[];
for(const type of process.platform==='darwin'?['dmg']:['deb','appimage']){
  const names=await readdir(path.join(bundle,type)).catch(error=>{if(error.code==='ENOENT')return [];throw error;});
  for(const name of names) {
    if(!/\.(deb|AppImage|dmg)$/i.test(name))continue;
    await copyFile(path.join(bundle,type,name),path.join(output,name));assets.push(name);
  }
}
if(process.platform==='darwin'){
  const folder=path.join(output,'Accento.app');
  await cp(path.join(bundle,'macos','Accento.app'),folder,{recursive:true,verbatimSymlinks:true});
  const name=`Accento_${version}_${target==='universal-apple-darwin'?'universal':process.arch}_macos.zip`;
  execFileSync('ditto',['-c','-k','--keepParent',folder,path.join(output,name)]);assets.push(name);
}
if(!assets.length)throw new Error(`No bundles found in ${bundle}`);
const sums=[];
for(const name of assets.sort())sums.push(`${createHash('sha256').update(await readFile(path.join(output,name))).digest('hex')}  ${name}`);
await writeFile(path.join(output,'SHA256SUMS.txt'),sums.join('\n')+'\n');
console.log(`Local ${process.platform} packages: ${output}. Nothing published.`);
