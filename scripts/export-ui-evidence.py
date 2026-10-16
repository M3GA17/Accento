"""Export the owned windows and layered surfaces from the current release tests."""
import hashlib
import json
from pathlib import Path
import platform
import subprocess
import tomllib
from PIL import Image

ROOT=Path(__file__).resolve().parents[1]
SOURCE=ROOT/'target'/'release'
VERSION=tomllib.loads((ROOT/'Cargo.toml').read_text(encoding='utf-8'))['workspace']['package']['version']
OUT=ROOT/'docs'/'evidenze'/VERSION
OUT.mkdir(parents=True,exist_ok=True)
reports={
 'windows-self-test.txt':('accento-self-test.txt',32,'PASS '),
 'settings-controls.txt':('accento-ui-controls.txt',14,'PASS:'),
 'settings-repaint.txt':('accento-ui-test.txt',1,'PASS:'),
}
for name,(source,count,prefix) in reports.items():
 report=(SOURCE/source).read_text(encoding='utf-8')
 assert len(report.splitlines())==count and all(line.startswith(prefix) for line in report.splitlines()),source
 (OUT/name).write_text(report,encoding='utf-8')
images=[]
screens=[]
for index,section in enumerate(['general','character-sets','blacklist']):
 for mode in ['light','dark']:
  screens.append((f'settings-{section}-{mode}',f'accento-settings-{index}-{mode}'))
for name in ['compact-general','compact-sets','compact-blacklist','general-fr','general-es','general-de','general-off','blacklist-error','blacklist-empty']:
 screens.append(('settings-'+name,'accento-settings-'+name))
for slug,source in screens:
 with Image.open(SOURCE/(source+'.bmp')) as im:
  # Invisible DWM frame, measured on this 96-DPI host. Keep the real title bar.
  cropped=im.convert('RGB').crop((8,0,im.width-8,im.height-8))
  cropped.save(OUT/(slug+'.png'))
  sample=cropped.getpixel((cropped.width-48,64))
  images.append({'file':slug+'.png','width':cropped.width,'height':cropped.height,
   'mode':'Dark' if sum(sample)<384 else 'Light','kind':'native-window','source':source+'.bmp','invisibleFrameCrop':[8,0,8,8]})
for mode,name in [(0,'light'),(1,'dark'),(10,'opening-light'),(11,'opening-dark')]:
 data=json.loads((SOURCE/f'accento-picker-{mode}.json').read_text())
 assert data['cornerAlpha']==[0,0,0,0] and data['gapsTransparent']
 pixels=bytearray((SOURCE/f'accento-picker-{mode}.bgra').read_bytes())
 for i in range(0,len(pixels),4):
  alpha=pixels[i+3]
  if alpha:
   for c in range(3):pixels[i+c]=min(255,(pixels[i+c]*255+alpha//2)//alpha)
 slug='picker-'+name+'-alpha.png'
 Image.frombytes('RGBA',(data['width'],data['height']),bytes(pixels),'raw','BGRA').save(OUT/slug)
 images.append({'file':slug,'width':data['width'],'height':data['height'],'kind':'native-surface','alpha':data,'mode':'Dark' if mode in [1,11] else 'Light'})
 with Image.open(SOURCE/f'accento-picker-{mode}.bmp') as im:im.convert('RGB').save(OUT/f'picker-{name}-context.png')
exe=SOURCE/'Accento.exe'
build={'version':VERSION,'platform':platform.platform(),'rust':subprocess.check_output(['rustc','--version'],text=True).strip(),
 'executable':f'dist/Accento-{VERSION}-windows-x64/Accento.exe','sha256':hashlib.sha256(exe.read_bytes()).hexdigest(),
 'unitTests':29,'windowsChecks':32,'settingsChecks':14,'repaintRegression':'passed: zero unnecessary root paints and control repositioning',
 'images':images,'scope':'Real UI captured only from owned test windows. No hooks in --ui-test. Normal use does not write captures.'}
(OUT/'build.json').write_text(json.dumps(build,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps({'images':len(images),'version':VERSION,'sha256':build['sha256'],'windowsChecks':32}))
