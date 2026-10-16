// Rasterize the shared vector mark for Windows. Run after editing accento.svg.
import {chromium} from '@playwright/test';
import {readFile, mkdir, writeFile} from 'node:fs/promises';

const source = await readFile(new URL('../ui/public/brand/accento.svg', import.meta.url), 'utf8');
const sizes = [16, 20, 24, 32, 48, 64, 128, 256];
const browser = await chromium.launch();
try {
  const page = await browser.newPage({deviceScaleFactor: 1});
  const images = [];
  for (const size of sizes) {
    await page.setViewportSize({width: size, height: size});
    await page.setContent(`<style>html,body{margin:0;background:transparent}svg{display:block;width:100vw;height:100vh}</style>${source}`);
    images.push(await page.screenshot({omitBackground: true}));
  }
  const header = Buffer.alloc(6 + sizes.length * 16);
  header.writeUInt16LE(1, 2);
  header.writeUInt16LE(sizes.length, 4);
  let offset = header.length;
  for (let i = 0; i < sizes.length; i++) {
    const entry = 6 + i * 16;
    header[entry] = header[entry + 1] = sizes[i] === 256 ? 0 : sizes[i];
    header.writeUInt16LE(1, entry + 4);
    header.writeUInt16LE(32, entry + 6);
    header.writeUInt32LE(images[i].length, entry + 8);
    header.writeUInt32LE(offset, entry + 12);
    offset += images[i].length;
  }
  const folder = new URL('../crates/accento-windows/icons/', import.meta.url);
  await mkdir(folder, {recursive: true});
  await writeFile(new URL('Accento.ico', folder), Buffer.concat([header, ...images]));
  await writeFile(new URL('128x128.png', folder), images[sizes.indexOf(128)]);
  await writeFile(new URL('128x128@2x.png', folder), images[sizes.indexOf(256)]);
  const icnsChunks=[];
  for (const [size,type] of [[128,'ic07'],[256,'ic08'],[512,'ic09'],[1024,'ic10']]) {
    let png=images[sizes.indexOf(size)];
    if (!png) {
      await page.setViewportSize({width:size,height:size});
      png=await page.screenshot({omitBackground:true});
    }
    const chunk=Buffer.alloc(8);chunk.write(type);chunk.writeUInt32BE(png.length+8,4);icnsChunks.push(chunk,png);
  }
  const icnsHeader=Buffer.alloc(8);icnsHeader.write('icns');icnsHeader.writeUInt32BE(8+icnsChunks.reduce((sum,b)=>sum+b.length,0),4);
  await writeFile(new URL('Accento.icns',folder),Buffer.concat([icnsHeader,...icnsChunks]));
  console.log(`Accento icons: ICO, PNG and ICNS from the shared SVG`);
} finally {
  await browser.close();
}
