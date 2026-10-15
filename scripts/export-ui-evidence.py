"""Export only the owned UI captured by Accento.exe --self-test (requires Pillow)."""
import hashlib
import json
from pathlib import Path
import platform
import subprocess
from PIL import Image

ROOT=Path(__file__).resolve().parents[1]
SOURCE=ROOT/'target'/'release'
OUT=ROOT/'docs'/'evidenze'/'0.3.0'
OUT.mkdir(parents=True,exist_ok=True)
report=(SOURCE/'accento-self-test.txt').read_text(encoding='utf-8')
assert len(report.splitlines())==35 and all(line.startswith('PASS ') for line in report.splitlines())
(OUT/'windows-self-test.txt').write_text(report,encoding='utf-8')
screens=[
 ('settings-writing-light','accento-settings-light'),
 ('settings-appearance-light','accento-settings-appearance'),
 ('settings-appearance-dark','accento-settings-dark'),
 ('settings-languages','accento-settings-languages'),
 ('settings-applications-error','accento-settings-error'),
 ('settings-compact-writing','accento-settings-compact-0'),
 ('settings-compact-languages','accento-settings-compact-1'),
 ('settings-compact-appearance','accento-settings-compact-2'),
 ('settings-compact-applications','accento-settings-compact-3'),
 ('settings-compact-scrolled','accento-settings-compact-scrolled'),
]
images=[]
for slug,source in screens:
    with Image.open(SOURCE/(source+'.bmp')) as im:
        # The 96-DPI test host has 8 invisible DWM pixels left/right/bottom.
        # Preserve all client content and the real native title bar.
        cropped=im.convert('RGB').crop((8,0,im.width-8,im.height-8))
        cropped.save(OUT/(slug+'.png'))
        sample=cropped.getpixel((cropped.width-48,64))
        images.append({'file':slug+'.png','width':cropped.width,'height':cropped.height,
                       'mode':'Dark' if sum(sample)<384 else 'Light',
                       'kind':'native-window','source':source+'.bmp','invisibleFrameCrop':[8,0,8,8]})
for mode,name in enumerate(['light','dark','solid']):
    data=json.loads((SOURCE/f'accento-picker-{mode}.json').read_text())
    assert data['cornerAlpha']==[0,0,0,0] and data['gapsTransparent']
    pixels=bytearray((SOURCE/f'accento-picker-{mode}.bgra').read_bytes())
    for i in range(0,len(pixels),4):
        alpha=pixels[i+3]
        if alpha:
            for c in range(3):pixels[i+c]=min(255,(pixels[i+c]*255+alpha//2)//alpha)
    slug='picker-'+name+'-alpha.png'
    Image.frombytes('RGBA',(data['width'],data['height']),bytes(pixels),'raw','BGRA').save(OUT/slug)
    sample=(20*data['width']+100)*4
    images.append({'file':slug,'width':data['width'],'height':data['height'],'kind':'native-surface','alpha':data,
                   'mode':'Dark' if sum(pixels[sample:sample+3])<384 else 'Light'})
    with Image.open(SOURCE/f'accento-picker-{mode}.bmp') as im:im.convert('RGB').save(OUT/f'picker-{name}-context.png')
exe=SOURCE/'Accento.exe'
build={'version':'0.3.0','platform':platform.platform(),'rust':subprocess.check_output(['rustc','--version'],text=True).strip(),
       'executable':'dist/Accento-0.3.0-windows-x64/Accento.exe','sha256':hashlib.sha256(exe.read_bytes()).hexdigest(),
       'unitTests':21,'windowsChecks':35,'images':images,
       'scope':'Real UI from the owned test window. Popup backdrop is inside that owned window; normal use does not write captures.'}
(OUT/'build.json').write_text(json.dumps(build,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps({'images':len(images),'sha256':build['sha256'],'windowsChecks':35}))
