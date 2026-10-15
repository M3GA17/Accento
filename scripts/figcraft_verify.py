"""Verify the reopened native document and its real exported pixels. Needs Pillow."""
import hashlib
import json
from pathlib import Path
from PIL import Image

OUT = Path(__file__).resolve().parents[1] / 'design' / 'figcraft'

def walk(node):
    yield node
    for child in node.get('children', []):
        yield from walk(child)

def verify():
    manifest=json.loads((OUT/'accento.manifest.json').read_text(encoding='utf-8'))
    doc=json.loads((OUT/'accento.inspection.json').read_text(encoding='utf-8'))
    nodes={n['id']:n for page in doc['pages'] for n in walk(page)}
    assert hashlib.sha256((OUT/manifest['file']).read_bytes()).hexdigest()==manifest['sha256']
    assert [p['name'][:2] for p in doc['pages']]==[f'{i:02d}' for i in range(8)]
    assert sum(n['type']=='COMPONENT' for n in nodes.values())==manifest['mainComponents']
    assert sum(n['type']=='COMPONENT_SET' for n in nodes.values())==manifest['componentSets']
    links=[]
    for n in nodes.values():
        for interaction in n.get('interactions', []):
            for action in interaction.get('actions', []):
                if action['type']=='navigate':
                    assert action['dest'] in nodes, (n['id'],action)
                    assert nodes[action['dest']]['type']=='FRAME'
                    links.append({'source':n['id'],'destination':action['dest']})
    for page in doc['pages']:
        for flow in page.get('flows', []):
            assert flow['start'] in nodes
    popup_checks=[]
    picker_set=next(n for n in nodes.values() if n['type']=='COMPONENT_SET' and n['name']=='Picker')
    for master in picker_set['children']:
        assert not master.get('fills') and not master.get('strokes') and not master.get('effects')
        assert all(child['type']=='INSTANCE' for child in master['children'])
        texts=[n['characters'] for n in walk(master) if n['type']=='TEXT']
        assert all(len(t)==1 for t in texts),texts
        assert len(texts)==len(master['children'])
        assert master['height']==80 and master['width']==72*len(texts)+8
    for b in manifest['boards']:
        with Image.open(OUT/'previews'/b['preview']) as im:
            assert im.size==(b['width'],b['height']),b
            if b['kind']=='transparent-popup':
                assert im.mode=='RGBA'
                assert im.getpixel((0,0))[3]==0
                assert im.getpixel((im.width-1,0))[3]==0
                assert im.getpixel((0,im.height-1))[3]==0
                assert im.getpixel((im.width-1,im.height-1))[3]==0
                # A little shadow may enter the gap; an enclosing panel must not.
                gaps=[im.getpixel((76+i*72,40))[3] for i in range((im.width-8)//72-1)]
                assert all(alpha<=12 for alpha in gaps),gaps
                popup_checks.append({'file':b['preview'],'size':list(im.size),
                                     'cornerAlpha':0,'gapAlpha':gaps})
    glass_nodes=[n for n in nodes.values() if any(e['type']=='glass' for e in n.get('effects', []))]
    assert glass_nodes
    report={'result':'passed','designVersion':manifest['designVersion'],'nativeSha256':manifest['sha256'],
      'pages':len(doc['pages']),'boards':len(manifest['boards']),'inspectedNodes':len(nodes),
      'prototypeLinks':len(links),'flows':sum(len(p.get('flows',[])) for p in doc['pages']),
      'nativeGlassLayers':len(glass_nodes),'popupPixelChecks':popup_checks,
      'checks':['Native hash matches saved file','Page order and component inventory',
        'All navigation destinations and flow starts resolve','Popup masters contain only one-character candidates',
        'Popup master dimensions and unpainted roots','All PNG dimensions match the artboards',
        'Transparent PNG corners and gap pixels','Native glass effects survive save and reopen'],
      'visualReview':['Writing light/dark','Languages light + compact','Appearance light/dark + compact',
        'Exclusions compact + inline error','Solid popup dark','Writing context dark','Component library',
        'Foundations dark','Glass on white, dark and coloured contexts'],
      'scope':'Design artifact verification; no Rust runtime or Windows compositor test.'}
    (OUT/'verification.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
    print(json.dumps({k:v for k,v in report.items() if k not in ['popupPixelChecks','checks','visualReview']},ensure_ascii=False))
    return report

if __name__=='__main__':verify()
