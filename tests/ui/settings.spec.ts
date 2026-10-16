import {test,expect} from '@playwright/test';
import {createRequire} from 'node:module';
const require=createRequire(import.meta.url);
const languages:{code:string;name:string}[]=require('../../localization/languages.json');
const messages:Record<string,Record<string,string>>=require('../../localization/messages.json');
test.beforeEach(async({page})=>{await page.goto('/?preview=1');await expect(page.getByRole('heading',{name:'Generale',exact:true})).toBeVisible();});

test('General combines basic controls and character sets without a separate set page',async({page})=>{
  for(const colorScheme of ['light','dark'] as const){
    await page.emulateMedia({colorScheme});
    await expect(page.locator('html')).toHaveAttribute('data-accento-theme','dark');
  }
  await expect(page.getByRole('heading',{name:'Aspetto',exact:true})).toHaveCount(0);
  await expect(page.getByRole('navigation').getByRole('button')).toHaveCount(2);
  await expect(page.getByRole('navigation').getByRole('button',{name:'Set caratteri',exact:true})).toHaveCount(0);
  await expect(page.locator('.sets-panel')).toBeVisible();
  const delay=await page.locator('.delay-control .accento-input-shell').boundingBox();
  const language=await page.locator('.accento-language [data-accento-part=trigger]').boundingBox();
  expect(delay&&language&&Math.abs(delay.y-language.y)<1).toBeTruthy();
  await expect(page.getByText('Set caratteri attivi',{exact:true})).toHaveCount(0);
  await expect(page.getByRole('button',{name:'Vedi tutti i caratteri →'})).toHaveCount(0);
});

test('all national-language variants are searchable and translate the entire settings view',async({page})=>{
  test.setTimeout(60000);
  for(const language of languages){
    await page.locator('.accento-language [data-accento-part=trigger]').click();
    await page.getByRole('searchbox').fill(language.name);
    await page.getByRole('option',{name:language.name,exact:true}).click();
    await expect(page.locator('html')).toHaveAttribute('lang',language.code);
    const dictionary=messages[language.code];
    await expect(page.getByRole('heading',{name:dictionary.General,exact:true})).toBeVisible();
    await expect(page.getByRole('button',{name:dictionary.Save,exact:true})).toBeVisible();
    await expect(page.locator('html')).toHaveAttribute('data-accento-theme','dark');
    await expect(page.getByRole('heading',{name:dictionary.SpecialSets,exact:true})).toBeVisible();
    await expect(page.locator('[data-special-set="currency"]')).toHaveAccessibleName(dictionary.Currency);
    await page.getByRole('button',{name:dictionary.General,exact:true}).click();
  }
});

test('currency set is enabled initially, independent of languages, and supports save and discard',async({page})=>{
  const currency=page.locator('[data-special-set="currency"]');
  await expect(currency).toBeChecked();
  await page.getByText('Valute · Varianti',{exact:true}).click();
  await expect(page.locator('.currency-details')).toContainText('€');
  await expect(page.locator('.currency-details')).toContainText('₱');
  await currency.focus();await page.keyboard.press('Space');
  await expect(currency).not.toBeChecked();
  await expect(page.locator('[data-language="it"]')).toBeChecked();
  await page.getByRole('button',{name:'Annulla modifiche'}).click();
  await expect(currency).toBeChecked();
  await currency.click();await page.getByRole('button',{name:'Salva modifiche'}).click();
  await expect(page.getByRole('status')).toHaveText('Tutte le modifiche sono salvate');
  await page.getByRole('button',{name:'BlackList',exact:true}).click();
  await page.getByRole('button',{name:'Generale',exact:true}).click();
  await expect(currency).not.toBeChecked();
  await currency.click();await page.getByRole('button',{name:'Annulla modifiche'}).click();
  await expect(currency).not.toBeChecked();
});

test('compact set panel expands with keyboard and bulk selection preserves one language',async({page})=>{
  const all=page.getByRole('checkbox',{name:'Tutte disponibili',exact:true});
  await expect(all).toHaveJSProperty('indeterminate',true);
  await all.check();
  await expect(page.locator('[data-language]:checked')).toHaveCount(languages.length);
  await expect(page.locator('[data-special-set="currency"]')).toBeChecked();
  await expect(all).toHaveJSProperty('indeterminate',false);
  await page.getByRole('textbox',{name:'Cerca lingua'}).fill('Albanian');
  await expect(page.locator('[data-language]')).toHaveCount(1);
  await expect(page.locator('[data-language="sq"]')).toBeChecked();
  await page.getByRole('textbox',{name:'Cerca lingua'}).fill('');
  await all.uncheck();
  await expect(page.locator('[data-language]:checked')).toHaveCount(1);
  await expect(page.getByRole('alert')).toHaveText('Mantieni almeno un set attivo.');
  await expect(page.locator('[data-special-set="currency"]')).not.toBeChecked();
  const disclosure=page.locator('.sets-disclosure');
  await disclosure.focus();await page.keyboard.press('Enter');
  await expect(disclosure).toHaveAttribute('aria-expanded','false');
  await expect(page.locator('.language-sets')).toBeHidden();
  await page.keyboard.press('Enter');
  await expect(disclosure).toHaveAttribute('aria-expanded','true');
  await expect(page.locator('[data-language]:checked')).toHaveCount(1);
  await page.getByRole('button',{name:'Annulla modifiche'}).click();
  await expect(page.locator('[data-special-set="currency"]')).toBeChecked();
  await page.getByRole('textbox',{name:'Cerca lingua'}).fill('zzzzz');
  await expect(page.getByText('Nessuna lingua trovata',{exact:true})).toBeVisible();
});

test('set checkboxes adapt to three, two and one column without horizontal overflow',async({page})=>{
  for(const [width,columns] of [[1040,3],[640,2],[520,1]]){
    await page.setViewportSize({width,height:780});
    await expect(page.locator('body')).toHaveJSProperty('scrollWidth',width);
    const count=await page.locator('.language-sets .sets-grid').evaluate(element=>getComputedStyle(element).gridTemplateColumns.split(' ').length);
    expect(count).toBe(columns);
    await expect(page.locator('[data-language="it"]')).toBeChecked();
  }
});

test('Albanian selection is independent of interface language and can be saved',async({page})=>{
  await page.getByRole('button',{name:'Lingua dell’interfaccia: Italiano'}).click();
  await page.getByRole('searchbox').fill('Albanian');
  await page.getByRole('option',{name:'Shqip',exact:true}).click();
  await expect(page.getByRole('heading',{name:'Të përgjithshme',exact:true})).toBeVisible();
  await page.getByRole('button',{name:'Ruaj ndryshimet'}).click();
  await expect(page.getByRole('status')).toHaveText('Të gjitha ndryshimet janë ruajtur');
  await expect(page.locator('[data-language="it"]')).toBeChecked();
  await expect(page.locator('[data-language="sq"]')).not.toBeChecked();
});

test('character sets have search, digraphs, all scripts and preserve the last active set',async({page})=>{
  await expect(page.locator('[data-language]')).toHaveCount(languages.length);
  await page.locator('[data-language="it"]').click();
  await expect(page.getByRole('alert')).toHaveText('Mantieni almeno un set attivo.');
  await page.getByRole('textbox',{name:'Cerca lingua'}).fill('Albanian');
  await expect(page.locator('[data-language]')).toHaveCount(1);
  await expect(page.locator('[data-language="sq"]')).toHaveAttribute('aria-description',/dh/);
  await page.locator('[data-language="sq"]').click();
  await page.getByRole('textbox',{name:'Cerca lingua'}).fill('');
  for(const code of ['el','ru','hy','ka','cnr-Cyrl'])await page.locator(`[data-language="${code}"]`).click();
  await page.getByRole('button',{name:'Salva modifiche'}).click();
  for(const code of ['it','sq','el','ru','hy','ka','cnr-Cyrl'])await expect(page.locator(`[data-language="${code}"]`)).toBeChecked();
});

test('delay validation, discard and pause preserve a draft',async({page})=>{
  const delay=page.getByRole('textbox',{name:'Ritardo di apertura'});
  await delay.fill('20');await page.getByRole('button',{name:'Salva modifiche'}).click();
  await expect(page.getByText('Inserisci un ritardo tra 100 e 1500 ms.')).toBeVisible();
  await delay.fill('450');await page.getByRole('button',{name:'Spegni Accento'}).click();
  await expect(delay).toHaveValue('450');await expect(page.getByText('Accento spento',{exact:true})).toBeVisible();
  await page.getByRole('button',{name:'Annulla modifiche'}).click();
  await expect(delay).toHaveValue('300');await expect(page.getByRole('button',{name:'Salva modifiche'})).toBeDisabled();
});

test('excluded executables reject paths and duplicates, support save and removal',async({page})=>{
  await page.getByRole('button',{name:'BlackList',exact:true}).click();
  const field=page.getByRole('textbox',{name:'Aggiungi applicazione'});
  await field.fill('C:\\bad.exe');await page.getByRole('button',{name:'Aggiungi',exact:true}).click();
  await expect(page.getByText('Inserisci un solo nome .exe, senza percorso o virgole.')).toBeVisible();
  await field.fill(' Notepad.EXE ');await page.getByRole('button',{name:'Aggiungi',exact:true}).click();
  await expect(page.getByText('notepad.exe',{exact:true})).toBeVisible();
  await field.fill('notepad.exe');await page.getByRole('button',{name:'Aggiungi',exact:true}).click();
  await expect(page.getByText('Questa applicazione è già nella BlackList.')).toBeVisible();
  await page.getByRole('button',{name:'Salva modifiche'}).click();
  await page.getByRole('button',{name:'Rimuovi notepad.exe'}).click();
  await expect(page.getByText('Nessuna applicazione nella BlackList')).toBeVisible();
});

test('keyboard selector, compact dark layout and long-script content',async({page})=>{
  await page.setViewportSize({width:640,height:620});
  await page.locator('.accento-language [data-accento-part=trigger]').focus();await page.keyboard.press('Enter');
  await page.getByRole('searchbox').fill('English');await page.keyboard.press('ArrowDown');await page.keyboard.press('Enter');
  await expect(page.getByRole('heading',{name:'General',exact:true})).toBeVisible();
  await expect(page.locator('body')).toHaveJSProperty('scrollWidth',640);
  await page.screenshot({path:'docs/evidenze/europe/settings-compact-dark.png'});
  await page.setViewportSize({width:1040,height:780});
  await page.locator('main').evaluate(element=>{element.scrollTop=0;});
  await page.screenshot({path:'docs/evidenze/europe/settings-general-dark.png'});
  await page.screenshot({path:'docs/evidenze/europe/settings-character-sets.png'});
});

test('dark settings and language popup have no serious accessibility violations',async({page})=>{
  await page.addScriptTag({path:require.resolve('axe-core/axe.min.js')});
  for(const section of ['General','Popup']){
    if(section==='Popup')await page.locator('.accento-language [data-accento-part=trigger]').click();
    const violations=await page.evaluate(async()=>{
      const result=await (window as any).axe.run(document,{runOnly:{type:'tag',values:['wcag2a','wcag2aa','wcag21aa']}});
      return result.violations.filter((item:any)=>['serious','critical'].includes(item.impact));
    });
    expect(violations).toEqual([]);await page.keyboard.press('Escape');
  }
});


test('Linux and macOS exclusions accept native process names and reject paths',async({page})=>{
  for(const [platform,name]of [['linux','firefox'],['macos','Safari']]){
    await page.goto(`/?preview=1&platform=${platform}`);
    await page.getByRole('button',{name:'BlackList',exact:true}).click();
    const field=page.getByRole('textbox',{name:'Aggiungi applicazione'});
    await expect(field).toHaveAttribute('placeholder',name);
    await field.fill(name);await page.getByRole('button',{name:'Aggiungi',exact:true}).click();
    await expect(page.getByText(name.toLowerCase(),{exact:true})).toBeVisible();
    await field.fill('/usr/bin/editor');await page.getByRole('button',{name:'Aggiungi',exact:true}).click();
    await expect(page.getByText('Inserisci un nome di processo, senza percorsi, virgole o caratteri jolly.',{exact:true})).toBeVisible();
    await page.getByRole('button',{name:'Salva modifiche'}).click();
    await expect(page.getByRole('status')).toHaveText('Tutte le modifiche sono salvate');
  }
});
