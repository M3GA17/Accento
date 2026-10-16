import {spawn} from 'node:child_process';
import {copyFile, mkdir, mkdtemp, readFile, writeFile} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import assert from 'node:assert/strict';
import {chromium} from '@playwright/test';

const {version} = JSON.parse(await readFile('package.json', 'utf8'));
const messages = JSON.parse(await readFile('localization/messages.json', 'utf8')).en;
const output = 'docs/assets';
await mkdir(output, {recursive: true});
const directory = await mkdtemp(path.join(tmpdir(), 'accento-docs-'));
const executable = path.resolve(process.argv[2] ?? 'target/release/Accento.exe');
const child = spawn(executable, ['--desktop-test'], {windowsHide: true, env: {
  ...process.env, LOCALAPPDATA: directory, ACCENTO_CONFIG_DIR: path.join(directory, 'Accento'),
  WEBVIEW2_USER_DATA_FOLDER: path.join(directory, 'WebView2'),
  WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: '--remote-debugging-port=9440',
}});
let desktop;
try {
  for (let attempt = 0; attempt < 100; attempt++) {
    if (child.exitCode !== null) throw new Error(`Desktop exited with ${child.exitCode}`);
    try {desktop = await chromium.connectOverCDP('http://127.0.0.1:9440'); break;}
    catch {await new Promise(resolve => setTimeout(resolve, 200));}
  }
  assert.ok(desktop, 'Isolated WebView2 connection unavailable');
  const context = desktop.contexts()[0];
  const page = context.pages()[0] ?? await context.waitForEvent('page');
  await page.getByRole('heading', {name: 'Generale', exact: true}).waitFor();
  assert.ok(!page.url().includes(':1421'), 'Documentation must capture the embedded release UI');
  await page.evaluate(async () => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const snapshot = await invoke('get_settings');
    await invoke('save_settings', {preferences: {...snapshot.preferences, ui_language: 'en'}});
  });
  await page.getByRole('heading', {name: messages.General, exact: true}).waitFor();
  assert.equal(await page.locator('.version').textContent(), `Accento ${version}`);
  await page.waitForFunction(() => document.querySelector('.brand-mark')?.naturalWidth > 0);
  await page.screenshot({path: `${output}/settings.png`});
  await page.getByRole('button', {name: messages.Sets, exact: true}).click();
  await page.screenshot({path: `${output}/character-sets.png`});
  await page.getByRole('button', {name: messages.General, exact: true}).click();
  await page.setViewportSize({width: 640, height: 620});
  await page.screenshot({path: `${output}/compact.png`});
} finally {
  await desktop?.close().catch(() => {});
  child.kill();
}

const data = async (file, type = 'image/png') => `data:${type};base64,${(await readFile(file)).toString('base64')}`;
const logo = await data('ui/public/brand/accento.svg', 'image/svg+xml');
const settings = await data(`${output}/settings.png`);
const picker = await data(`${output}/picker-source.png`);
const browser = await chromium.launch();
try {
  const page = await browser.newPage({viewport: {width: 1400, height: 900}, deviceScaleFactor: 1});
  await page.setContent(`<!doctype html><style>
    *{box-sizing:border-box}body{margin:0;font-family:'Segoe UI',sans-serif;color:#f2f5fb;background:#0b1019}
    .hero{position:relative;width:1400px;height:900px;overflow:hidden;background:radial-gradient(ellipse at 90% 0%,#23374d88,transparent 60%),radial-gradient(ellipse at 0% 100%,#26304766,transparent 55%)}
    .brand{position:absolute;left:64px;top:54px;display:flex;gap:14px;align-items:center;font-size:28px;font-weight:600;letter-spacing:-1px}.brand img{width:62px;height:62px}
    .badge{position:absolute;right:64px;top:73px;color:#afbed4;font-size:13px;letter-spacing:2px}
    h1{position:absolute;left:68px;top:176px;margin:0;font-size:66px;font-weight:550;letter-spacing:-3.2px;line-height:1.08}
    .intro{position:absolute;left:70px;top:421px;width:400px;color:#a9b7c9;font-size:20px;line-height:1.65}
    .window{position:absolute;left:558px;top:169px;width:778px;border:1px solid #61728a40;border-radius:16px;overflow:hidden;box-shadow:0 32px 90px #0006;background:#101010}
    .bar{height:35px;display:flex;align-items:center;gap:7px;padding:0 13px;background:#1c2028;color:#a9b7c9;font-size:11px}.dot{width:6px;height:6px;border-radius:50%;background:#667384}.bar span:last-child{margin:auto}
    .window>img{display:block;width:100%}.sample{position:absolute;left:58px;top:564px;width:304px;height:160px}.hint{position:absolute;left:72px;top:739px;font-size:14px;color:#97a8be}.hint strong{color:#dce6f4;font-weight:500}
    .footer{position:absolute;bottom:46px;left:72px;font-size:15px;color:#91a1b8;letter-spacing:.3px}.footer b{color:#d7e1ef;font-weight:500}
  </style><div class='hero'><div class='brand'><img src='${logo}'>Accento</div><div class='badge'>BETA 0.5 / WINDOWS</div>
    <h1>A little accent.<br>A smoother<br>sentence.</h1><p class='intro'>Accents and character variants,<br>right where you type.</p>
    <div class='window'><div class='bar'><i class='dot'></i><i class='dot'></i><i class='dot'></i><span>Accento · Settings</span></div><img src='${settings}'></div>
    <img class='sample' src='${picker}'><div class='hint'><strong>Hold e</strong> &nbsp; + &nbsp; <strong>tap Space</strong> &nbsp; → &nbsp; choose & release</div>
    <div class='footer'><b>47 language entries</b> &nbsp; · &nbsp; Native glass keys &nbsp; · &nbsp; Works offline</div></div>`);
  await page.screenshot({path: `${output}/hero.png`});
  await page.setViewportSize({width: 900, height: 280});
  await page.setContent(`<style>body{margin:0;font-family:'Segoe UI',sans-serif;color:#e9eff8;background:#121a27;display:flex;align-items:center;justify-content:center;height:280px;gap:52px}img{width:304px;height:160px}h2{font-size:26px;font-weight:500;letter-spacing:-.6px;margin:0 0 12px}p{font-size:15px;color:#a5b6ce;line-height:1.7;margin:0}small{display:block;font-size:11px;color:#7387a3;margin-top:18px}</style><img src='${picker}'><div><h2>Choose. Release. Keep writing.</h2><p>Floating glass keys.<br>Your focus stays in the document.</p><small>Native rendering fixture · displayed at 2× size</small></div>`);
  await page.screenshot({path: `${output}/picker.png`});
} finally {
  await browser.close();
}
await writeFile(`${output}/badges.svg`, `<svg xmlns="http://www.w3.org/2000/svg" width="590" height="32" viewBox="0 0 590 32"><rect width="590" height="32" rx="16" fill="#1c2635"/><g fill="#d1ddec" font-family="Segoe UI,sans-serif" font-size="12" text-anchor="middle"><text x="61" y="21">0.5 BETA</text><text x="199" y="21">Windows 11 x64</text><text x="346" y="21">Rust + Tauri</text><text x="471" y="21">47 entries</text><text x="555" y="21">MIT</text></g></svg>\n`);
await copyFile('ui/public/brand/accento.svg', `${output}/logo.svg`);
console.log(`README artwork captured from Accento ${version} with an isolated profile. No publication performed.`);
