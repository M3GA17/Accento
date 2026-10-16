import {spawn} from 'node:child_process';
import {mkdtemp,readFile,mkdir,writeFile} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import assert from 'node:assert/strict';
import {chromium} from 'playwright';

// A separate mutex, no keyboard hook, and an isolated preferences directory protect the running app.
const directory=await mkdtemp(path.join(tmpdir(),'accento-desktop-'));
const executable=path.resolve(process.argv[2]??'target/debug/Accento.exe');
const child=spawn(executable,['--desktop-test'],{windowsHide:true,env:{...process.env,LOCALAPPDATA:directory,ACCENTO_CONFIG_DIR:path.join(directory,'Accento'),WEBVIEW2_USER_DATA_FOLDER:path.join(directory,'WebView2'),WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:'--remote-debugging-port=9437'}});
const errors=[];child.stderr.on('data',data=>errors.push(data.toString()));
let browser;
try{
  for(let attempt=0;attempt<100;attempt++){
    if(child.exitCode!==null)throw new Error(`Desktop exited: ${child.exitCode}\n${errors.join('')}`);
    try{browser=await chromium.connectOverCDP('http://127.0.0.1:9437');break;}catch{await new Promise(resolve=>setTimeout(resolve,200));}
  }
  assert.ok(browser,'WebView2 debugging connection unavailable');
  const context=browser.contexts()[0];
  let page=context.pages()[0]??await context.waitForEvent('page');
  await page.waitForFunction(()=>!!window.__TAURI_INTERNALS__);
  await page.getByRole('heading',{name:'Generale',exact:true}).waitFor();
  const loadedUrl=page.url();
  if(executable.replaceAll('\\','/').includes('/release/'))assert.ok(!loadedUrl.includes(':1421'),'Release must use embedded assets, not the development server');
  const invoke=(command,args)=>page.evaluate(({command,args})=>window.__TAURI_INTERNALS__.invoke(command,args),{command,args});
  const initial=await invoke('get_settings');assert.equal(initial.preferences.delay_ms,300);
  assert.equal(initial.preferences.theme,'dark');
  const languages=JSON.parse(await readFile('localization/languages.json','utf8'));
  for(const language of languages){
    const next={...initial.preferences,ui_language:language.code,languages:[language.code],theme:'light'};
    const snapshot=await invoke('save_settings',{preferences:next});
    assert.equal(snapshot.preferences.ui_language,language.code);
    assert.deepEqual(snapshot.preferences.languages,[language.code]);
    assert.equal(snapshot.preferences.theme,'dark','Legacy or forged light preferences must not change dark mode');
  }
  const invalid={...initial.preferences,delay_ms:2};
  await assert.rejects(()=>invoke('save_settings',{preferences:invalid}));
  assert.equal((await invoke('get_settings')).preferences.delay_ms,300);
  const preferences={...initial.preferences,delay_ms:450,ui_language:'sq',languages:['sq','ro','pl','tr','pt'],excluded_apps:['notepad.exe'],theme:'dark'};
  const saved=await invoke('save_settings',{preferences});assert.deepEqual(saved.preferences,preferences);
  await page.getByRole('heading',{name:'Të përgjithshme',exact:true}).waitFor();
  await page.getByRole('button',{name:'Grupet e shkronjave',exact:true}).waitFor();
  const disk=JSON.parse(await readFile(path.join(directory,'Accento/settings.json'),'utf8'));assert.deepEqual(disk,preferences);
  assert.equal((await invoke('set_paused',{paused:true})).paused,true);
  await page.getByRole('button',{name:'Aktivizo Accento'}).waitFor();
  assert.equal((await invoke('set_paused',{paused:false})).paused,false);
  await mkdir('docs/evidenze/react-tauri',{recursive:true});
  await page.screenshot({path:'docs/evidenze/react-tauri/desktop-albanian-dark.png'});
  // Real Tauri close interception: a draft must survive the attempted close.
  await page.getByRole('textbox',{name:'Vonesa e hapjes'}).fill('600');
  await invoke('plugin:window|close');
  await page.getByRole('alertdialog').waitFor();
  await page.getByRole('button',{name:'Vazhdo redaktimin',exact:true}).click();
  assert.equal(await page.getByRole('textbox',{name:'Vonesa e hapjes'}).inputValue(),'600');
  await page.getByRole('button',{name:'Anulo ndryshimet',exact:true}).click();
  await page.getByRole('button',{name:'Ruaj ndryshimet'}).waitFor({state:'visible'});
  await page.waitForFunction(()=>document.querySelector('footer button[data-variant="primary"]')?.disabled);
  const closed=page.waitForEvent('close');
  await invoke('plugin:window|close');
  await closed;
  assert.equal(child.exitCode,null,'Closing settings must leave the backend running');
  await writeFile('docs/evidenze/react-tauri/desktop-test.json',JSON.stringify({passed:true,executable,loadedUrl,locales:languages.length,checks:['Tauri/WebView2 startup','Rust get/save/pause IPC','invalid configuration rejected without changing state','all national locales and character sets accepted','dark mode enforced for legacy/forged light preferences','atomic persisted preferences','React receives native state events','unsaved draft close protection','webview closes while backend remains running'],keyboardHook:false},null,2)+'\n');
  console.log(`Desktop integration: 9 checks and ${languages.length} locale/set round-trips passed. Keyboard hooks disabled for this isolated test.`);
}finally{
  await browser?.close().catch(()=>{});
  child.kill();
}
