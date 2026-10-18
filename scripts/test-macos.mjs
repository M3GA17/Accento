import assert from 'node:assert/strict';
import {execFileSync, spawn} from 'node:child_process';
import {createHash} from 'node:crypto';
import {mkdtemp, readFile, readdir, rm, writeFile} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';

if (process.platform !== 'darwin') throw new Error('Run the packaged Mac verification on macOS.');
const {version} = JSON.parse(await readFile('package.json', 'utf8'));
const app = path.resolve(process.argv[2] ?? `dist/releases/v${version}/macos/Accento.app`);
const executable = path.join(app, 'Contents/MacOS/Accento');
const resources = path.join(app, 'Contents/Resources');
const architectures = execFileSync('lipo', ['-archs', executable], {encoding: 'utf8'}).trim().split(/\s+/);
assert.ok(architectures.includes('arm64') && architectures.includes('x86_64'), 'Both Mac architectures must be present.');
execFileSync('codesign', ['--verify', '--deep', '--strict', app], {stdio: 'inherit'});
const diagnostics = JSON.parse(execFileSync(executable, ['--diagnostics'], {encoding: 'utf8'}));
assert.equal(diagnostics.os, 'macos');
assert.equal(diagnostics.version, version);
assert.equal(typeof diagnostics.permissions.accessibility, 'boolean');
const bundleVersion = execFileSync('/usr/libexec/PlistBuddy', ['-c', 'Print :CFBundleShortVersionString', path.join(app, 'Contents/Info.plist')], {encoding: 'utf8'}).trim();
assert.ok([version, version.split('-')[0]].includes(bundleVersion), 'Bundle version must match the beta.');
const guides = (await readdir(path.join(resources, 'guide'))).filter(name => name.endsWith('.md'));
assert.equal(guides.length, 47);
for (const name of ['README.txt', 'TEST-MACOS.it.txt', 'LICENSE', 'THIRD-PARTY-NOTICES.md', 'PLATFORMS.md']) {
  assert.ok((await readFile(path.join(resources, name))).length > 0, `Missing resource: ${name}`);
}

const profile = await mkdtemp(path.join(tmpdir(), 'accento-macos-'));
const reportPath = path.join(profile, 'desktop.json');
const child = spawn(executable, ['--desktop-test'], {
  env: {...process.env, ACCENTO_CONFIG_DIR: profile, ACCENTO_DESKTOP_REPORT: reportPath},
  stdio: ['ignore', 'pipe', 'pipe'],
});
let output = '';
child.stdout.on('data', data => { output += data; });
child.stderr.on('data', data => { output += data; });
try {
  const code = await new Promise((resolve, reject) => {
    const timeout = setTimeout(() => { child.kill(); reject(new Error('Native WebView verification timed out after 30 seconds.')); }, 30000);
    child.once('error', error => { clearTimeout(timeout); reject(error); });
    child.once('exit', code => { clearTimeout(timeout); resolve(code); });
  });
  assert.equal(code, 0, `App exited with ${code}. ${output}`);
  const desktop = JSON.parse(await readFile(reportPath, 'utf8'));
  assert.equal(desktop.passed, true, JSON.stringify(desktop));
  assert.equal(desktop.platform.os, 'macos');
  assert.equal(desktop.keyboardHook, false);
  const preferences = JSON.parse(await readFile(path.join(profile, 'settings.json'), 'utf8'));
  assert.equal(preferences.delay_ms, 450);
  assert.equal(preferences.ui_language, 'en');
  assert.equal(preferences.currency_symbols, false);
  const verification = {
    version, bundleVersion, architectures, runtimeArchitecture: process.arch, diagnostics,
    macOS: execFileSync('sw_vers', ['-productVersion'], {encoding: 'utf8'}).trim(),
    executableSha256: createHash('sha256').update(await readFile(executable)).digest('hex'),
    signatureVerified: true, guides: guides.length, desktop, preferencesPersisted: true,
    liveInputVerified: false, permissionFlowVerified: false,
  };
  await writeFile(path.join(path.dirname(app), 'macos-verification.json'), JSON.stringify(verification, null, 2) + '\n');
  console.log('Universal Mac bundle, signature, resources, native WebView, IPC and isolated persistence verified. Live input and permissions await the tester.');
} catch (error) {
  console.error(output);
  console.error(await readFile(reportPath, 'utf8').catch(() => 'No native desktop report was produced.'));
  throw error;
} finally {
  if (child.exitCode === null) child.kill();
  await rm(profile, {recursive: true, force: true});
}
