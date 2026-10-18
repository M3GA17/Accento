import {test, expect} from '@playwright/test';

// Preserve the previous UI exactly, with intentional additions for Currency.
for (const viewport of [{width:1040,height:780},{width:640,height:620}]) {
  test(`settings appearance ${viewport.width}`, async ({page}) => {
    test.skip(process.platform !== 'win32', 'Pixel baselines use Windows fonts and rendering.');
    await page.setViewportSize(viewport);
    await page.goto('/?preview=1');
    await expect(page.getByRole('heading',{name:'Generale',exact:true})).toBeVisible();
    const capture = async (name:string) => {
      await page.mouse.move(0,0);
      await expect.poll(()=>page.locator('.brand-mark').evaluate(element=>{
        const image=element as HTMLImageElement;
        return image.complete && image.naturalWidth>0;
      })).toBe(true);
      // Windows can round antialiased border colors by one RGB level between frames.
      await expect(page).toHaveScreenshot(`${viewport.width}-${name}.png`,{maxDiffPixels:0,threshold:0.01});
    };
    await capture('general');
    await page.getByRole('textbox',{name:'Ritardo di apertura'}).fill('20');
    await page.getByRole('button',{name:'Salva modifiche'}).click();
    await expect(page.getByText('Inserisci un ritardo tra 100 e 1500 ms.')).toBeVisible();
    await capture('delay-error');
    await page.getByRole('button',{name:'Annulla modifiche'}).click();
    await page.getByRole('button',{name:'Lingua dell’interfaccia: Italiano'}).click();
    await expect(page.getByRole('searchbox')).toBeVisible();
    await capture('language-popup');
    await page.getByRole('searchbox').fill('Albanian');
    await expect(page.getByRole('option')).toHaveCount(1);
    await capture('language-search');
    await page.getByRole('searchbox').fill('zzzz');
    await expect(page.getByText('Nessuna lingua trovata',{exact:true})).toBeVisible();
    await capture('language-empty');
    await page.goto('/?preview=1');
    await expect(page.getByRole('heading',{name:'Generale',exact:true})).toBeVisible();
    if(viewport.width===1040){
      await page.setViewportSize({width:1040,height:1440});
      await page.locator('.sets-panel').screenshot({path:'docs/evidenze/europe/character-sets-panel.png'});
      await page.screenshot({path:'docs/evidenze/europe/settings-general-combined.png'});
      await page.setViewportSize(viewport);
    }
    await page.locator('.sets-disclosure').click();
    await capture('sets-collapsed');
    await page.locator('.sets-disclosure').click();
    await page.getByRole('checkbox',{name:'Tutte disponibili',exact:true}).check();
    await capture('sets-all');
    await page.getByRole('button',{name:'Annulla modifiche'}).click();
    await page.getByText('Valute · Varianti',{exact:true}).click();
    await capture('currency-keys');
    await page.getByText('Valute · Varianti',{exact:true}).click();
    await page.getByRole('textbox',{name:'Cerca lingua'}).fill('Albanian');
    await page.locator('[data-language="sq"]').click();
    await capture('sets-selected');
    await page.getByRole('button',{name:'BlackList',exact:true}).click();
    await capture('blacklist-empty');
    await page.getByRole('textbox',{name:'Aggiungi applicazione'}).fill('C:\\bad.exe');
    await page.getByRole('button',{name:'Aggiungi',exact:true}).click();
    await capture('blacklist-error');
    await page.getByRole('textbox',{name:'Aggiungi applicazione'}).fill('notepad.exe');
    await page.getByRole('button',{name:'Aggiungi',exact:true}).click();
    await capture('blacklist-entry');
  });
}
