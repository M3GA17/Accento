import {spawnSync} from 'node:child_process';
import path from 'node:path';
const args=process.argv.slice(2);
const run=(command,args=[])=>{
  const result=spawnSync(command,args,{stdio:'inherit',shell:process.platform==='win32'});
  if(result.error)throw result.error;
  if(result.status!==0)process.exit(result.status??1);
};
if(process.platform==='win32'){
  if(args.length)throw new Error('Use scripts/build.ps1 for Windows packaging options.');
  run('powershell',['-NoProfile','-ExecutionPolicy','Bypass','-File','scripts/build.ps1']);
}else{
  run('npm',['run','build']);run('npm',['run','resources:prepare']);
  run('npm',['run','test:ui']);
  run('cargo',['fmt','--all','--','--check']);
  run('cargo',['test','--workspace','--locked']);
  run('cargo',['clippy','--workspace','--all-targets','--locked','--','-D','warnings']);
  run(process.execPath,[path.resolve('node_modules/@tauri-apps/cli/tauri.js'),'build','--ci',...args,'--','--locked']);
  run(process.execPath,['scripts/package-unix.mjs',...args]);
}
