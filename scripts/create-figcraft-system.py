"""Editable Accento product UI, design 0.3. Runtime changes are a separate step.
Run: python -X utf8 scripts/create-figcraft-system.py
"""
import hashlib
import json
from figcraft_mcp import MCP, OUT

TOKENS = {
 'Light': {'canvas':'#F7F8FA','sidebar':'#EBEFF480','surface':'#FFFFFF','ink':'#202633',
   'secondary':'#657083','tertiary':'#7C8798','accent':'#275CDD','selection':'#336BE914',
   'selection-rim':'#3C73ED85','line':'#24355016','inset':'#F0F2F6','rim':'#FFFFFFE8',
   'inner-rim':'#52637D35','glass-start':'#FFFFFF82','glass-middle':'#F5F9FF26',
   'glass-end':'#D9E5F64D','hover':'#3767C90D','success':'#218450','error':'#B52D44','on-accent':'#FFFFFF'},
 'Dark': {'canvas':'#161A22','sidebar':'#20283480','surface':'#202630','ink':'#EFF3FA',
   'secondary':'#A4AFC0','tertiary':'#8895A9','accent':'#8AB6FF','selection':'#7BA9FF1C',
   'selection-rim':'#8EB8FF85','line':'#D5E4FF18','inset':'#141A23','rim':'#FFFFFF70',
   'inner-rim':'#00000066','glass-start':'#A8BEDB38','glass-middle':'#40536E19',
   'glass-end':'#1C2A434D','hover':'#D9E9FF0C','success':'#79DBA3','error':'#FF9BAC','on-accent':'#FFFFFF'}
}
METRICS = {'space/1':4,'space/2':8,'space/3':12,'space/4':16,'space/6':24,'space/8':32,
 'radius/candidate':17,'radius/group':16,'radius/control':10,'radius/window':16,
 'size/candidate':64,'gap/candidate':8,'size/picker-height':80,'size/field':40,
 'size/row':72,'size/sidebar':216,'size/settings-width':1000,'size/settings-height':740,
 'stroke/edge':1,'stroke/focus':2,'time/open-default-ms':300}
SECTIONS = ['Scrittura','Lingue','Aspetto','Applicazioni']
DESCRIPTIONS = {'Scrittura':'Il tuo gesto, al ritmo giusto.', 'Lingue':'Solo i caratteri che ti servono.',
 'Aspetto':'La luce cambia. La scrittura resta al centro.',
 'Applicazioni':'Scegli dove lasciare la tastiera così com’è.'}
GLASS = {'type':'glass','depth':12,'refraction':.65,'dispersion':.16,'frost':3,
         'lightAngle':-48,'lightIntensity':.72,'splay':.1}
PATHS = {
 'Scrittura':'M 3 5 L 17 5 M 10 5 L 10 18 M 6 18 L 14 18',
 'Lingue':'M 10 1 C -2 1 -2 19 10 19 C 22 19 22 1 10 1 Z M 0 10 L 20 10 M 10 1 C 4 5 4 15 10 19 M 10 1 C 16 5 16 15 10 19',
 'Aspetto':'M 10 1 C -2 1 -2 19 10 19 C 22 19 22 1 10 1 Z M 10 1 L 10 19',
 'Applicazioni':'M 1 2 L 19 2 L 19 18 L 1 18 Z M 1 7 L 19 7 M 5 4 L 5 5 M 8 4 L 8 5',
 'check':'M 3 10 L 8 15 L 17 5','close':'M 5 5 L 15 15 M 15 5 L 5 15',
 'plus':'M 10 4 L 10 16 M 4 10 L 16 10','minus':'M 4 10 L 16 10'}

def rgba(value):
    value=value.lstrip('#')
    channels=[int(value[i:i+2],16)/255 for i in range(0,len(value),2)]
    return dict(zip(('r','g','b','a'),channels+([1] if len(channels)==3 else [])))

class System:
    def __init__(self):
        OUT.mkdir(parents=True,exist_ok=True)
        (OUT/'previews').mkdir(exist_ok=True)
        self.m=MCP()
        self.m.command('file.new',name='Accento / Interfacce finali · Liquid Glass')
        self.collection=self.m.command('variable.createCollection',name='Accento / Appearance',modes=['Light','Dark'])['id']
        self.variables={}
        for role in TOKENS['Light']:
            self.variables[role]=self.m.command('variable.create',collection=self.collection,name='color/'+role,
                type='color',values={m:v[role] for m,v in TOKENS.items()})['id']
        for role,value in METRICS.items():
            self.variables[role]=self.m.command('variable.create',collection=self.collection,name=role,
                type='number',values={'Light':value,'Dark':value})['id']
        self.pages=[]; self.boards=[]; self.components={}; self.sets={}; self.styles={}
        self.screens={}; self.popups=[]

    def bind(self,node,field,role):
        self.m.command('variable.bind',ids=[node],field=field,variable=self.variables[role])
        if field in ('fill','stroke'):
            self.m.command('fill.set' if field=='fill' else 'stroke.set',ids=[node],index=0,opacity=1)

    def node(self,typ,p,name,x,y,w,h,role=None,**props):
        if role:props['fill']=TOKENS['Light'][role]
        extra={'path':props.pop('path')} if 'path' in props else {}
        node=self.m.call('create_node',type=typ,parent=p,name=name,x=x,y=y,width=w,height=h,props=props,**extra)['id']
        if role:self.bind(node,'fill',role)
        return node

    def set(self,node,**props):
        return self.m.call('set_properties',ids=[node],props=props)

    def rect(self,p,n,x,y,w,h,role=None,r=0,**props):
        return self.node('rectangle',p,n,x,y,w,h,role,cornerRadius=r,**props)

    def text(self,p,t,x,y,w,size=14,role='ink',bold=False,h=None,align='left'):
        height=h or size*1.5
        n=self.node('text',p,t,x,y,w,height,role,characters=t,fontFamily='Segoe UI Variable',
           fontStyle='Semibold' if bold else 'Regular',fontSize=size,lineHeight=size*1.4,
           textAutoResize='none',textAlign=align,textAlignVertical='middle')
        key=f'Accento/{size}/'+('Semibold' if bold else 'Regular')
        if key not in self.styles:
            self.styles[key]=self.m.command('style.create',type='text',name=key,
                value={'fontFamily':'Segoe UI Variable','fontStyle':'Semibold' if bold else 'Regular',
                       'fontSize':size,'lineHeight':size*1.4})['id']
        self.m.command('style.apply',ids=[n],style=self.styles[key])
        self.set(n,width=w,height=height,textAutoResize='none',textAlign=align,textAlignVertical='middle')
        return n

    def page(self,name):
        if not self.pages:
            p=self.m.call('inspect_document')['pages'][0]['id']
            self.m.command('page.rename',id=p,name=name)
        else:p=self.m.command('page.add',name=name)['id']
        self.m.command('page.select',id=p)
        self.m.command('page.setBackground',color='#353B47')
        self.pages.append(p)
        return p

    def frame(self,p,name,mode,x,y,w,h,transparent=False,radius=0):
        props={'fills':[]} if transparent else {}
        n=self.node('frame',p,name,x,y,w,h,None if transparent else 'canvas',clip=True,cornerRadius=radius,**props)
        self.m.command('variable.setMode',ids=[n],collection=self.collection,mode=mode)
        return n

    def export_board(self,n,slug,mode,w,h,kind='spec'):
        self.boards.append({'id':n,'name':slug,'mode':mode,'width':w,'height':h,'kind':kind})

    def board(self,p,title,mode,x=0,y=0,w=1200,h=900,slug=None):
        b=self.frame(p,title+' / '+mode,mode,x,y,w,h)
        self.text(b,'ACCENTO   /   SPECIFICA DI PRODOTTO',48,32,w-96,11,'secondary',True)
        self.text(b,title,48,72,w-96,28,bold=True,h=44)
        self.text(b,'15 ottobre 2026 · Design 0.3 · '+mode,48,h-36,w-96,11,'tertiary')
        self.export_board(b,slug or title.lower().replace(' ','-'),mode,w,h)
        return b

    def outline(self,p,x,y,w,h,r=10,role='line',weight=1):
        n=self.rect(p,'Bordo / '+role,x,y,w,h,r=r,fills=[],stroke=TOKENS['Light'][role],strokeWeight=weight)
        self.bind(n,'stroke',role)
        return n

    def line(self,p,x,y,w):return self.rect(p,'Separatore',x,y,w,1,'line')

    def glass(self,p,x,y,w,h,r=17,selected=False,state='default',opaque=False):
        if opaque:self.rect(p,'Superficie solida',x,y,w,h,'surface',r)
        else:
            roles=['glass-start','glass-middle','glass-end']
            surface=self.rect(p,'Liquid Glass / rifrazione nativa',x,y,w,h,r=r,
                fills=[{'type':'gradient','kind':'linear','from':[0,0],'to':[.8,1],
                        'stops':[{'position':pos,'color':rgba(TOKENS['Light'][role])}
                                 for pos,role in zip([0,.52,1],roles)]}],
                effects=[{'type':'dropShadow','color':'#07162E20','radius':8,'offsetX':0,'offsetY':3},GLASS])
            for i,role in enumerate(roles):self.bind(surface,f'fills/0/stops/{i}/color',role)
        if selected:self.rect(p,'Tinta della selezione',x,y,w,h,'selection',r)
        if state=='hover':self.rect(p,'Hover',x,y,w,h,'hover',r)
        self.outline(p,x+.5,y+.5,w-1,h-1,r-.5,'rim')
        self.outline(p,x+1.5,y+1.5,w-3,h-3,r-1.5,'selection-rim' if selected else 'inner-rim',1 if selected else .6)
        if state=='focus':self.outline(p,x-3,y-3,w+6,h+6,r+3,'accent',2)

    def vector(self,p,name,x,y,path,role='secondary',size=20):
        n=self.node('vector',p,name,x,y,size,size,path=path,fills=[],stroke=TOKENS['Light'][role],
                    strokeWeight=1.5,strokeCap='round',strokeJoin='round')
        self.bind(n,'stroke',role)
        return n

    def icon(self,p,kind,x,y,role='secondary'):return self.vector(p,kind,x,y,PATHS[kind],role)

    def component(self,p,family,key,x,y,w,h):
        c=self.node('component',p,key,x,y,w,h,fills=[],clip=False,
                    description=f'Accento design 0.3 / {family}; implementation status in handoff.json')
        self.components[(family,key)]=c
        return c

    def property(self,c,name,node,value):
        self.m.command('component.addProperty',id=c,name=name,type='text',default=value,layer=node)

    def use(self,family,key,p,x,y,**properties):
        n=self.m.command('instance.create',component=self.components[(family,key)],parent=p,x=x,y=y)['id']
        if properties:self.m.command('instance.setProperties',ids=[n],properties=properties)
        return n

    def combine(self,family,ids):
        self.m.command('component.combineVariants',ids=ids)
        inventory=self.m.command('component.list')
        n=next(c for c in reversed(inventory) if c['type']=='COMPONENT_SET' and c['name'] not in self.sets)
        self.set(n['id'],name=family,clip=False,fills=[],strokes=[])
        self.sets[family]=n['id']

    def library(self):
        p=self.page('04 / Componenti')
        b=self.board(p,'Controlli alla scala del prodotto','Light',w=1160,h=1640,slug='componenti')
        self.text(b,'Candidati · 64 × 64 · separazione trasparente 8',48,152,1040,16,bold=True)
        ids=[]
        for i,(key,on,state,solid) in enumerate([('Rest',False,'default',False),('Selected',True,'default',False),
            ('Hover',False,'hover',False),('Focus',True,'focus',False),('Solid',False,'default',True),('SolidSelected',True,'default',True)]):
            c=self.component(b,'Candidate',key,48+i*172,204,64,64)
            self.glass(c,0,0,64,64,selected=on,state=state,opaque=solid)
            t=self.text(c,'è',0,0,64,32,'accent' if on else 'ink',h=64,align='center')
            self.property(c,'Glyph',t,'è'); ids.append(c)
            self.text(b,key,48+i*172,284,160,12,'secondary')
        self.combine('Candidate',ids)
        self.text(b,'Navigazione · una destinazione alla volta',48,356,1040,16,bold=True)
        ids=[]
        for i,key in enumerate(['Rest','Selected','Hover','Focus']):
            c=self.component(b,'Navigation',key,48+(i%2)*520,404+(i//2)*76,184,44)
            if key!='Rest':self.rect(c,'Stato',0,0,184,44,'selection' if key in ['Selected','Focus'] else 'hover',10)
            self.icon(c,'Scrittura',12,12,'accent' if key in ['Selected','Focus'] else 'secondary')
            t=self.text(c,'Scrittura',44,0,126,14,'accent' if key in ['Selected','Focus'] else 'ink',key in ['Selected','Focus'],h=44)
            self.property(c,'Label',t,'Scrittura')
            if key=='Focus':self.outline(c,-2,-2,188,48,12,'accent',2)
            self.text(b,key,264+(i%2)*520,416+(i//2)*76,200,12,'secondary'); ids.append(c)
        self.combine('Navigation',ids)
        self.text(b,'Switch · target 48 × 40',48,592,450,16,bold=True)
        ids=[]
        for i,key in enumerate(['Off','On','Focus']):
            c=self.component(b,'Switch',key,48+i*172,640,48,40)
            self.rect(c,'Track',2,7,44,26,'accent' if key!='Off' else 'tertiary',13)
            self.node('ellipse',c,'Thumb',24 if key!='Off' else 5,10,20,20,fill='#FFFFFF',
                effects=[{'type':'dropShadow','color':'#00000026','radius':2,'offsetX':0,'offsetY':1}])
            if key=='Focus':self.outline(c,-2,3,52,34,17,'accent',2)
            ids.append(c); self.text(b,key,48+i*172,692,160,12,'secondary')
        self.combine('Switch',ids)
        self.text(b,'Azioni · 144 × 40',632,592,450,16,bold=True)
        ids=[]
        for i,key in enumerate(['Primary','Secondary','Disabled']):
            c=self.component(b,'Button',key,632,640+i*68,144,40)
            if key=='Primary':self.rect(c,'Button',0,0,144,40,r=10,fill='#2F64DB')
            else:
                self.rect(c,'Button',0,0,144,40,'inset',10,opacity=.55 if key=='Disabled' else 1)
                self.outline(c,0,0,144,40)
            label='Annulla' if key=='Secondary' else 'Salva modifiche'
            t=self.text(c,label,0,0,144,13,'on-accent' if key=='Primary' else 'tertiary' if key=='Disabled' else 'ink',True,h=40,align='center')
            self.property(c,'Label',t,label); ids.append(c)
            self.text(b,key,800,650+i*68,250,12,'secondary')
        self.combine('Button',ids)
        self.text(b,'Campi · valori reali, errori vicini al dato',48,832,1040,16,bold=True)
        ids=[]
        for i,key in enumerate(['Rest','Focus','Error']):
            c=self.component(b,'Field',key,48+i*360,884,320,72)
            self.rect(c,'Input',0,0,320,40,'inset',10)
            self.outline(c,0,0,320,40,10,'error' if key=='Error' else 'accent' if key=='Focus' else 'line',1.5 if key!='Rest' else 1)
            value='notepad' if key=='Error' else 'notepad.exe'
            t=self.text(c,value,12,0,296,13,h=40); self.property(c,'Value',t,value)
            if key=='Error':self.text(c,'Inserisci un nome completo: app.exe',0,46,320,12,'error',h=22)
            for child in self.m.call('get_node',id=c,depth=1)['children']:
                self.set(child['id'],constraints={'horizontal':'leftRight','vertical':'top'})
            ids.append(c); self.text(b,key,48+i*360,970,320,12,'secondary')
        self.combine('Field',ids)
        self.text(b,'Lingue · riga compatta, selezione multipla',48,1042,1040,16,bold=True)
        ids=[]
        for i,key in enumerate(['Off','On','Focus']):
            c=self.component(b,'LanguageRow',key,48,1090+i*104,640,72)
            self.rect(c,'Row',0,0,640,72,'surface',12)
            if key!='Off':self.rect(c,'Selection',0,0,640,72,'selection',12)
            self.rect(c,'Language monogram',16,16,40,40,'inset',10)
            t=self.text(c,'à',16,16,40,24,h=40,align='center'); self.property(c,'Glyph',t,'à')
            t=self.text(c,'Italiano',72,12,340,14,bold=True,h=24); self.property(c,'Label',t,'Italiano')
            t=self.text(c,'à  è  é  ì  ò  ó  ù',72,38,460,12,'secondary',h=20); self.property(c,'Preview',t,'à  è  é  ì  ò  ó  ù')
            self.rect(c,'Checkbox',598,26,20,20,'accent' if key!='Off' else 'inset',6)
            if key!='Off':self.icon(c,'check',598,26,'on-accent')
            else:self.outline(c,598,26,20,20,6)
            if key=='Focus':self.outline(c,-2,-2,644,76,14,'accent',2)
            for child in self.m.call('get_node',id=c,depth=1)['children']:
                if child['x']>=598:
                    self.set(child['id'],constraints={'horizontal':'right','vertical':'top'})
                elif child['name'] in ['Row','Selection'] or (child['type']=='TEXT' and child['x']==72):
                    self.set(child['id'],constraints={'horizontal':'leftRight','vertical':'top'})
            ids.append(c); self.text(b,key,728,1114+i*104,300,12,'secondary')
        self.combine('LanguageRow',ids)
        self.text(b,'Il glass appartiene ai controlli flottanti. I gruppi delle impostazioni restano quieti.',48,1460,1060,16,'secondary',h=54)
        print('Library: candidate, navigation, switch, button, field, language rows',flush=True)

    def picker(self,p,x,y,glyphs='èé',selected=0,opaque=False,name='Popup'):
        w=len(glyphs)*72+8
        c=self.component(p,'Picker',name,x,y,w,80)
        for i,glyph in enumerate(glyphs):
            key=('SolidSelected' if i==selected else 'Solid') if opaque else ('Selected' if i==selected else 'Rest')
            self.use('Candidate',key,c,8+i*72,8,Glyph=glyph)
        self.popups.append({'id':c,'glyphs':glyphs,'selected':selected,'opaque':opaque})
        return c

    def popup_library(self):
        p=self.page('01 / Popup trasparente')
        b=self.board(p,'Solo la lettera. Tutto il resto scompare.','Light',w=1160,h=950,slug='popup-specifica')
        self.text(b,'NESSUN PANNELLO ESTERNO · NESSUNA LABEL NEL POPUP',48,144,1060,12,'secondary',True)
        ids=[]
        for i,(name,glyphs,selected,opaque) in enumerate([
            ('Apertura','èé',-1,False),('Selezione','èé',0,False),('Maiuscole','ÈÉ',1,False),
            ('Combinato','èéêë',2,False),('Singolo','à',0,False),('Solido','èé',0,True)]):
            xx,yy=48+(i%2)*548,216+(i//2)*210
            self.text(b,name,xx,yy,500,14,bold=True)
            ids.append(self.picker(b,xx,yy+42,glyphs,selected,opaque,name))
        self.combine('Picker',ids)
        self.text(b,'64 × 64 DIP · gap 8 · margine tecnico 8 · altezza 80 · larghezza 72n + 8',48,854,1060,13,'secondary')
        for i,mode in enumerate(TOKENS):
            for j,(slug,key,w) in enumerate([('popup-apertura','Apertura',152),('popup-selezione','Selezione',152),
                                           ('popup-combinato','Combinato',296),('popup-solido','Solido',152)]):
                root=self.frame(p,key+' / '+mode,mode,1260+i*420,j*164,w,80,True)
                self.use('Picker',key,root,0,0)
                self.export_board(root,slug,mode,w,80,'transparent-popup')
        print('Transparent popup masters + light/dark exports',flush=True)

    def group(self,p,x,y,w,h):
        self.rect(p,'Gruppo',x,y,w,h,'surface',16)
        self.outline(p,x+.5,y+.5,w-1,h-1,15.5)

    def keycap(self,p,label,x,y,w=40):
        self.rect(p,'Tasto '+label,x,y,w,36,'inset',9); self.outline(p,x,y,w,36,9)
        self.text(p,label,x,y,w,13,bold=True,h=36,align='center')

    def shell(self,p,section,mode,x,y,state='clean',width=1000,height=740):
        c=self.frame(p,f'{section} / {mode} / {state}',mode,x,y,width,height,radius=16)
        compact=width<900; rail=0 if compact else 216
        left=32 if compact else rail+40; cw=width-left-(32 if compact else 40)
        self.text(c,'Accento',20,0,200,12,'secondary',h=40)
        for i,xx in enumerate([width-136,width-90,width-44]):
            path=['M 3 10 L 13 10','M 3 3 L 13 3 L 13 13 L 3 13 Z','M 3 3 L 13 13 M 13 3 L 3 13'][i]
            self.vector(c,['Riduci','Ingrandisci','Chiudi'][i],xx+14,12,path,size=16)
        controls={}
        if not compact:
            self.rect(c,'Sidebar / materiale',0,40,216,height-40,'sidebar')
            self.rect(c,'Mark',24,72,32,32,r=10,fill='#2F64DB')
            self.text(c,'á',24,70,32,22,'on-accent',True,h=34,align='center')
            self.text(c,'Accento',68,74,124,20,bold=True,h=28)
            self.text(c,'IMPOSTAZIONI',24,138,168,10,'secondary',True)
            for i,label in enumerate(SECTIONS):
                n=self.use('Navigation','Selected' if label==section else 'Rest',c,16,174+i*52,Label=label)
                if label!='Scrittura':
                    tree=self.m.call('get_node',id=n,depth=1)
                    glyph=next(k for k in tree['children'] if k['type']=='VECTOR')
                    self.set(glyph['id'],visible=False)
                    self.icon(c,label,28,186+i*52,'accent' if label==section else 'secondary')
                controls[label]=n
            self.line(c,24,height-126,168)
            self.node('ellipse',c,'Attivo',24,height-96,6,6,'success')
            self.text(c,'Accento è attivo',38,height-107,154,12,'secondary',h=28)
            self.text(c,'Resta nell’area di notifica',24,height-73,172,11,'tertiary',h=24)
        else:
            self.line(c,0,40,width)
            for i,label in enumerate(SECTIONS):
                n=self.node('frame',c,'Tab '+label,32+i*cw/4,56,cw/4,40,fills=[])
                if label==section:self.rect(n,'Tab attivo',0,0,cw/4-8,40,'selection',10)
                self.text(n,label,0,0,cw/4-8,13,'accent' if label==section else 'secondary',label==section,h=40,align='center')
                controls[label]=n
        top=132 if compact else 82
        self.text(c,section,left,top,cw,28,bold=True,h=42)
        self.text(c,DESCRIPTIONS[section],left,top+46,cw,13,'secondary',h=24)
        footer=height-80; self.line(c,rail,footer,width-rail)
        status,role={'dirty':('Modifiche da salvare','secondary'),'saved':('Impostazioni salvate','success'),
            'error':('Controlla il campo evidenziato','error')}.get(state,('Tutte le modifiche sono salvate','secondary'))
        self.text(c,status,left,footer+21,cw-170,12,role,h=40)
        controls['Save']=self.use('Button','Primary' if state in ['dirty','error'] else 'Disabled',c,width-184,footer+20)
        return c,controls,left,top+98,cw,footer

    def settings(self,p,section,mode,x,y,state='clean',width=1000,height=740,appearance='Automatico',material=True):
        c,controls,l,top,w,footer=self.shell(p,section,mode,x,y,state,width,height)
        if section=='Scrittura':
            self.text(c,'ATTIVAZIONE',l,top,w,11,'secondary',True,h=20)
            gy=top+32; self.group(c,l,gy,w,164)
            self.text(c,'Gesto di apertura',l+20,gy+18,w-40,14,bold=True,h=24)
            self.keycap(c,'Lettera',l+20,gy+54,72); self.text(c,'+',l+102,gy+58,24,14,'secondary')
            self.keycap(c,'Spazio',l+132,gy+54,76)
            self.text(c,'Tieni la lettera, premi Spazio e scegli con ← →.',l+20,gy+108,w-40,13,'secondary',h=22)
            self.text(c,'Rilascia la lettera per inserire. Esc per annullare.',l+20,gy+132,w-40,12,'secondary',h=20)
            gy+=180; self.group(c,l,gy,w,100)
            self.text(c,'Ritardo di apertura',l+20,gy+16,w-224,14,bold=True,h=24)
            self.text(c,'Da 100 a 1500 ms',l+20,gy+49,w-224,12,'secondary',h=24)
            self.rect(c,'Ritardo',l+w-174,gy+20,154,40,'inset',10)
            self.text(c,'300',l+w-128,gy+20,60,14,bold=True,h=40,align='center')
            self.text(c,'ms',l+w-96,gy+60,56,10,'tertiary',h=20,align='center')
            controls['Minus']=self.icon(c,'minus',l+w-162,gy+30)
            controls['Plus']=self.icon(c,'plus',l+w-52,gy+30)
            gy+=124; self.text(c,'PROVALO QUI',l,gy,w,11,'secondary',True,h=20)
            self.group(c,l,gy+30,w,80)
            self.text(c,'Scrivi una parola…',l+20,gy+47,w-40,16,'tertiary',h=34)
        elif section=='Lingue':
            self.text(c,'LINGUE DISPONIBILI',l,top,w,11,'secondary',True,h=20)
            self.text(c,'1 attiva',l+w-100,top,100,12,'secondary',h=20,align='right')
            gy=top+32
            for i,(glyph,label,preview) in enumerate([('à','Italiano','à  è  é  ì  ò  ó  ù'),
                ('ç','Français','à  â  æ  ç  é  è  ê  ë  î  ï  ô  œ  ù  û  ü  ÿ'),
                ('ñ','Español','á  é  í  ñ  ó  ú  ü'),('ü','Deutsch','ä  ö  ü  ß')]):
                n=self.use('LanguageRow','On' if i==0 else 'Off',c,l,gy+i*82,Glyph=glyph,Label=label,Preview=preview)
                if w!=640:
                    self.set(n,width=w)
                controls[label]=n
            self.text(c,'Puoi attivare più lingue. I caratteri comuni compaiono una sola volta.',l,gy+348,w,12,'secondary',h=42)
            self.text(c,'Mantieni almeno una lingua attiva.',l,gy+394,w,12,'secondary',h=22)
        elif section=='Aspetto':
            self.text(c,'TEMA',l,top,w,11,'secondary',True,h=20)
            gy=top+32; self.group(c,l,gy,w,146); tilew=(w-48)/3
            for i,label in enumerate(['Automatico','Chiaro','Scuro']):
                n=self.node('frame',c,'Tema '+label,l+12+i*(tilew+12),gy+12,tilew,122,fills=[])
                selected=label==appearance
                self.rect(n,'Theme choice',0,0,tilew,122,'selection' if selected else 'canvas',12)
                if selected:self.outline(n,.5,.5,tilew-1,121,11.5,'selection-rim')
                sw=tilew-32
                self.rect(n,'Anteprima tema',16,12,sw,58,r=7,fill='#202632' if label=='Scuro' else '#F6F8FC',stroke='#8896AC44',strokeWeight=.7)
                if label=='Automatico':self.rect(n,'Metà scura',16+sw/2,12,sw/2,58,r=[0,7,7,0],fill='#202632')
                self.rect(n,'Mini rail',22,20,18,42,r=4,fill='#8594AB40')
                for j in range(3):self.rect(n,'Mini contenuto',48,22+j*12,sw-48,4,r=2,fill='#8795AB66')
                self.text(n,label,8,82,tilew-16,13,'accent' if selected else 'ink',selected,h=28,align='center')
                controls[label]=n
            self.text(c,'Automatico segue il tema di Windows.' if appearance=='Automatico' else 'Il tema cambia subito in anteprima. Salva per conservarlo.',l,gy+156,w,12,'secondary',h=22)
            gy+=208; self.text(c,'MATERIALE DEL POPUP',l,gy,w,11,'secondary',True,h=20)
            self.group(c,l,gy+32,w,90)
            self.text(c,'Liquid Glass',l+20,gy+46,w-108,14,bold=True,h=24)
            self.text(c,'Luce e trasparenza sui singoli caratteri.',l+20,gy+74,w-108,12,'secondary',h=22)
            controls['Glass']=self.use('Switch','On' if material else 'Off',c,l+w-68,gy+56)
            self.text(c,'Con la trasparenza disattivata, i tasti usano una superficie solida.',l,gy+138,w-192,12,'secondary',h=48)
            self.use('Picker','Selezione' if material else 'Solido',c,l+w-160,gy+132)
        else:
            self.text(c,'APP ESCLUSE',l,top,w,11,'secondary',True,h=20)
            gy=top+32; self.group(c,l,gy,w,220)
            self.text(c,'Dove Accento non si attiva',l+20,gy+16,w-40,14,bold=True,h=24)
            self.text(c,'Nomi completi, separati da virgola. Senza percorsi.',l+20,gy+48,w-40,12,'secondary',h=24)
            field=self.use('Field','Error' if state=='error' else 'Focus' if state=='dirty' else 'Rest',c,l+20,gy+94,
                Value='notepad' if state=='error' else 'notepad.exe, WindowsTerminal.exe')
            self.set(field,width=w-40)
            controls['Exclusions']=field
            self.text(c,'Il confronto usa il nome del processo e ignora maiuscole e minuscole.',l+20,gy+178,w-40,12,'secondary',h=24)
            self.text(c,'Nelle app escluse la tastiera mantiene il comportamento abituale.',l,gy+244,w,13,'secondary',h=48)
        return c,controls

    def link(self,node,dest,trigger=None):
        args={'id':node,'action':{'type':'navigate','dest':dest},'transition':{'type':'instant'}}
        if trigger:args['trigger']=trigger
        self.m.command('proto.interaction.add',**args)

    def settings_screens(self):
        p=self.page('02 / Impostazioni finali')
        for mi,mode in enumerate(TOKENS):
            for i,section in enumerate(SECTIONS):
                c,controls=self.settings(p,section,mode,i*1080,mi*840)
                self.screens[(mode,section)]=(c,controls)
                self.export_board(c,'impostazioni-'+section.lower(),mode,1000,740,'product-screen')
        for (mode,section),(c,controls) in self.screens.items():
            for target in SECTIONS:self.link(controls[target],self.screens[(mode,target)][0])
            if section=='Aspetto':
                self.link(controls['Chiaro'],self.screens[('Light','Aspetto')][0])
                self.link(controls['Scuro'],self.screens[('Dark','Aspetto')][0])
        for mode in TOKENS:self.m.command('proto.flow.add',start=self.screens[(mode,'Scrittura')][0],name='Impostazioni / '+mode)
        compact={}
        for i,section in enumerate(SECTIONS):
            c,controls=self.settings(p,section,'Light',i*800,1740,width=720,height=820)
            compact[section]=(c,controls)
            self.export_board(c,'compatta-'+section.lower(),'Light',720,820,'product-screen')
        for section,(c,controls) in compact.items():
            for target in SECTIONS:self.link(controls[target],compact[target][0])
        print('Settings: 4 sections in both modes + compact reflow',flush=True)

    def states(self):
        p=self.page('03 / Stati e feedback')
        for i,state in enumerate(['dirty','error','saved']):
            c,controls=self.settings(p,'Applicazioni','Light',i*1080,0,state)
            self.screens[('Light',state)]=(c,controls)
            self.export_board(c,'stato-'+state,'Light',1000,740,'product-state')
            for target in SECTIONS:self.link(controls[target],self.screens[('Light',target)][0])
        self.link(self.screens[('Light','dirty')][1]['Save'],self.screens[('Light','saved')][0])
        self.link(self.screens[('Light','error')][1]['Exclusions'],self.screens[('Light','dirty')][0])
        self.m.command('proto.flow.add',start=self.screens[('Light','error')][0],name='Correggi e salva / dimostrazione')
        # Explicit choices have truthful selected controls; theme and material changes
        # are reviewable visual states, with dirty -> saved transitions.
        theme_states={}
        for i,(mode,appearance,material) in enumerate([
            ('Light','Chiaro',True),('Dark','Scuro',True),('Light','Chiaro',False),('Dark','Scuro',False)]):
            key=('solido-' if not material else '')+mode.lower()
            for j,state in enumerate(['dirty','saved']):
                c,controls=self.settings(p,'Aspetto',mode,i*1080,860+j*840,state,
                                         appearance=appearance,material=material)
                theme_states[(key,state)]=(c,controls)
                self.export_board(c,'aspetto-'+key+'-'+state,mode,1000,740,'product-state')
                for target in SECTIONS:self.link(controls[target],self.screens[(mode,target)][0])
        for (key,state),(c,controls) in theme_states.items():
            self.link(controls['Chiaro'],theme_states[('light','dirty')][0])
            self.link(controls['Scuro'],theme_states[('dark','dirty')][0])
            mode='Dark' if key.endswith('dark') else 'Light'
            self.link(controls['Automatico'],self.screens[(mode,'Aspetto')][0])
            material_target=mode.lower() if key.startswith('solido-') else 'solido-'+mode.lower()
            self.link(controls['Glass'],theme_states[(material_target,'dirty')][0])
            if state=='dirty':self.link(controls['Save'],theme_states[(key,'saved')][0])
        for mode in TOKENS:
            controls=self.screens[(mode,'Aspetto')][1]
            for label,key in [('Chiaro','light'),('Scuro','dark')]:
                self.m.command('proto.interaction.clear',ids=[controls[label]])
                self.link(controls[label],theme_states[(key,'dirty')][0])
            self.link(controls['Glass'],theme_states[('solido-'+mode.lower(),'dirty')][0])

    def context(self,p,mode,x,y):
        c=self.frame(p,'Durante la scrittura / '+mode,mode,x,y,1000,650)
        self.rect(c,'Documento di contesto',0,0,1000,650,'surface'); self.line(c,0,46,1000)
        self.text(c,'Note',24,0,600,13,'secondary',h=46)
        self.text(c,'Una giornata a Milano',136,138,750,32,bold=True,h=52)
        self.text(c,'Il cielo è chiaro, le strade sono ancora tranquille.',136,232,750,20,'secondary',h=40)
        self.text(c,'Ci fermiamo per un caffe',136,292,750,20,h=40)
        self.rect(c,'Cursore nel destinatario',367,300,1.5,26,'accent')
        self.text(c,'La città si sveglia, una parola alla volta.',136,370,750,20,'secondary',h=40)
        self.use('Picker','Selezione',c,424,534)
        return c

    def product(self):
        p=self.pages[0]; self.m.command('page.select',id=p)
        for i,mode in enumerate(TOKENS):
            c,controls=self.settings(p,'Scrittura',mode,i*1080,0)
            self.export_board(c,'prodotto-scrittura',mode,1000,740,'product-screen')
            for target in SECTIONS:self.link(controls[target],self.screens[(mode,target)][0])
            self.m.command('proto.flow.add',start=c,name='Prodotto / '+mode)
            c=self.context(p,mode,i*1080,860)
            self.export_board(c,'prodotto-popup-in-uso',mode,1000,650,'context')
        b=self.board(p,'Il materiale segue ciò che c’è sotto.','Light',0,1630,2080,560,'glass-su-sfondi')
        for i,(name,fill) in enumerate([('Documento chiaro','#FBFCFE'),('Editor scuro','#1B2330'),('Colore e contrasto',None)]):
            xx=48+i*662; self.text(b,name,xx,150,610,14,bold=True)
            if fill:self.rect(b,'Sfondo di verifica',xx,198,610,250,r=16,fill=fill)
            else:
                self.rect(b,'Sfondo di verifica',xx,198,610,250,r=16,fills=[{'type':'gradient','kind':'linear','from':[0,0],'to':[1,1],
                  'stops':[{'position':0,'color':rgba('#AEC8C6')},{'position':.48,'color':rgba('#DFB896')},{'position':1,'color':rgba('#8CA4CF')}]}])
                for j in range(7):self.rect(b,'Linee del contesto',xx+52+j*76,226,20,194,r=10,fill='#FFFFFF24')
            root=self.frame(b,'Materiale su '+name,'Dark' if i==1 else 'Light',xx+229,282,152,80,True)
            self.use('Picker','Selezione',root,0,0)
        print('Product page: complete settings + writing context + glass on 3 backdrops',flush=True)

    def foundations(self):
        p=self.page('05 / Materiale e token')
        for i,mode in enumerate(TOKENS):
            b=self.board(p,'Trasparenza, gerarchia, misura.',mode,i*1280,w=1200,h=1380,slug='fondamenti')
            self.text(b,'Una superficie quieta per le impostazioni. Vetro sui controlli flottanti.',48,144,1104,16,'secondary',h=48)
            for j,role in enumerate(TOKENS[mode]):
                xx,yy=48+(j%5)*224,230+(j//5)*132
                self.rect(b,role,xx,yy,192,48,role,10)
                self.text(b,role,xx,yy+54,200,12,bold=True); self.text(b,TOKENS[mode][role],xx,yy+78,200,11,'secondary')
            self.text(b,'La gerarchia usa dimensioni e spazio, non cornici ripetute.',48,916,1104,24,bold=True,h=76)
            for j,(label,value) in enumerate([('Titolo / 28','Scrittura'),('Riga / 14','Ritardo di apertura'),('Supporto / 12','Da 100 a 1500 ms')]):
                xx=48+j*368; self.text(b,label,xx,1040,336,12,'secondary')
                self.text(b,value,xx,1080,336,[28,14,12][j],bold=j<2,h=48)
            self.text(b,'Glass nativo FigCraft: depth 12 · refraction 0,65 · dispersion 0,16 · frost 3.',48,1180,1104,13,'secondary',h=32)
            self.text(b,'Tasti 64 · gap 8 · raggio 17. Gruppi R16. Campi e pulsanti alti 40. Focus 2.',48,1224,1104,13,'secondary',h=32)
            self.text(b,'Il runtime 0.3 riprende la geometria; il materiale Windows usa un compositore proprio con alpha per pixel.',48,1268,1104,12,'secondary',h=40)

    def handoff(self):
        p=self.page('06 / Interazione e consegna')
        b=self.board(p,'Dal prodotto progettato al prodotto implementato.','Light',w=1280,h=1190,slug='consegna')
        rows=[('Popup','Radice senza fill, bordo o backdrop. Solo tasti indipendenti. Zero label o istruzioni.'),
          ('Dimensioni','64 × 64 per tasto; gap 8; margine tecnico 8. Larghezza 72n + 8, altezza 80 DIP.'),
          ('Apertura','Lettera trattenuta + Spazio, attesa configurabile. All’apertura nessun candidato selezionato.'),
          ('Scelta','← → o nuova pressione di Spazio. Tinta e bordo distinguono la scelta, oltre al colore del glifo.'),
          ('Conferma','Rilascio della lettera base. Esc annulla. Il focus rimane nel programma destinatario.'),
          ('Posizione','In basso al centro del monitor attivo. Clamp all’area utile. Il popup non segue ancora il cursore.'),
          ('Impostazioni','Scrittura → Lingue → Aspetto → Applicazioni. Footer stabile e salvataggio esplicito.'),
          ('Compatta','A 720 DIP la navigazione passa in alto. Testo a misura invariata; niente riduzione uniforme.'),
          ('Validazione','Input preservato, errore vicino al campo. Almeno una lingua. Ritardo 100–1500 ms.'),
          ('Materiale','Vetro rifrattivo sui candidati; impostazioni sobrie. Superficie solida quando la trasparenza è ridotta.')]
        for i,(title,body) in enumerate(rows):
            yy=166+i*76; self.text(b,f'{i+1:02d}',48,yy,40,11,'accent',True,h=40)
            self.text(b,title,104,yy,220,15,bold=True,h=40); self.text(b,body,340,yy,892,13,'secondary',h=52)
            self.line(b,48,yy+64,1184)
        self.text(b,'Stato della consegna',48,956,1184,22,bold=True,h=40)
        self.text(b,'Il runtime Rust 0.3 implementa questa UI. La pagina 08 contiene le catture reali.\nIl glass Windows usa un backdrop campionato all’apertura e composizione alpha; non il motore ottico di FigCraft.',48,1012,1184,14,'secondary',h=70)
        self.text(b,'Prototipo: navigazione delle 4 sezioni, cambio Light/Dark, correzione e salvataggio dimostrativo.',48,1096,1184,12,'secondary',h=40)

    def tray(self):
        p=self.page('07 / Area di notifica')
        for i,mode in enumerate(TOKENS):
            b=self.board(p,'Sempre disponibile, fuori dalla scrittura.',mode,i*1120,w=1040,h=640,slug='area-notifica')
            for j,paused in enumerate([False,True]):
                xx=48+j*488; self.text(b,'In pausa' if paused else 'Attivo',xx,166,440,14,bold=True)
                self.group(b,xx,214,264,156)
                self.text(b,'Impostazioni',xx+20,226,224,14,h=40)
                self.text(b,'Riprendi' if paused else 'Pausa',xx+20,272,224,14,h=40)
                self.line(b,xx+12,316,240); self.text(b,'Esci',xx+20,322,224,14,h=40)
            self.text(b,'Chiudere le impostazioni lascia Accento nell’area di notifica.',48,430,944,20,bold=True,h=64)
            self.text(b,'Il menu mostra i comandi reali. La sua cornice definitiva viene disegnata da Windows.',48,518,944,13,'secondary',h=48)

    def finish(self):
        self.m.command('page.select',id=self.pages[0]); self.m.call('select',ids=[])
        path=OUT/'Accento-Design-System.figcraft'
        self.m.call('save_file',path=str(path)); self.m.call('open_file',path=str(path))
        doc=self.m.call('inspect_document',depth=16,allPages=True)
        inventory=self.m.command('component.list')
        mains=[c for c in inventory if c['type']=='COMPONENT']; sets=[c for c in inventory if c['type']=='COMPONENT_SET']
        assert len(doc['pages'])==len(self.pages)
        assert len(mains)==len(self.components)
        assert {s['name'] for s in sets}==set(self.sets)
        assert len(doc['styles'])==len(self.styles)
        for popup in self.popups:
            n=self.m.call('get_node',id=popup['id'],depth=3)
            assert not n.get('fills') and not n.get('strokes') and not n.get('effects'),popup
            assert len(n.get('children',[]))==len(popup['glyphs'])
            assert all(child['type']=='INSTANCE' for child in n['children'])
        self.m.command('variable.export',path=str(OUT/'accento.variables.json'))
        variables=json.loads((OUT/'accento.variables.json').read_text(encoding='utf-8'))
        assert len(variables['meta']['variables'])==len(self.variables)
        (OUT/'accento.inspection.json').write_text(json.dumps(doc,ensure_ascii=False,indent=2),encoding='utf-8')
        expected=[]
        for b in self.boards:
            filename=f'{b["name"]}-{b["mode"].lower()}.png'; b['preview']=filename; expected.append(filename)
            self.m.call('export',ids=[b['id']],format='png',scale=1,includeBoundingBox=True,path=str(OUT/'previews'/filename))
        for old in (OUT/'previews').glob('*.png'):
            if old.name not in expected:old.unlink()
        manifest={'file':path.name,'designVersion':'0.3.0','runtimeVersion':'0.3.0',
          'sha256':hashlib.sha256(path.read_bytes()).hexdigest(),'pages':len(doc['pages']),
          'mainComponents':len(mains),'componentSets':len(sets),'variables':len(self.variables),'textStyles':len(self.styles),
          'boards':self.boards,'components':inventory,
          'verification':'Saved, reopened, inspected, all boards exported; popup roots checked transparent and label-free.',
          'limitations':['Windows glass uses a backdrop sampled once per gesture, not the FigCraft optical engine.',
             'Prototype navigation and save are demonstrations, not OS input or persistence.',
             'Native FigCraft glass does not prove Windows backdrop behaviour.']}
        (OUT/'accento.manifest.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2),encoding='utf-8')
        (OUT/'accento.tokens.json').write_text(json.dumps({'contract':'accento.glass/2','modes':TOKENS,'metrics':METRICS,
           'font':'Segoe UI Variable','unit':'Windows DIP at 96 DPI','glass':GLASS,
           'popup':{'background':'none','labels':False,'widthFormula':'72 * candidateCount + 8',
                    'height':80,'tileSize':64,'gap':8,'padding':8}},ensure_ascii=False,indent=2),encoding='utf-8')
        mapping={'designVersion':'0.3.0','runtimeVersion':'0.3.0','status':'implemented / Windows self-test verified',
          'sourceOfTruth':path.name,'unit':'DIP','implementedChanges':[
            {'file':'crates/accento-windows/src/render.rs','implemented':'Layered BGRA surface, transparent root; independent keys at x = 8 + 72i, y = 8; no visible labels.'},
            {'file':'crates/accento-windows/src/picker_material.rs','implemented':'Blurred and refracted backdrop sampled once per opening; solid alternative; alpha gaps at multiple DPI and text scales.'},
            {'file':'crates/accento-windows/src/app.rs','implemented':'Width 72n+8, height 80; no rectangular DWM backdrop; preserve no-activate and bottom-centre placement.'},
            {'file':'crates/accento-windows/src/app/settings.rs','implemented':'Four sections; 40-DIP controls; compact tabs below 900 DIP; real clipped viewport and pinned footer, without width-based scaling.'},
            {'file':'crates/accento-windows/src/glass.rs','implemented':'Quiet settings palette and shared drawing primitives; native title bar replaces the design title bar.'}],
          'evidence':'docs/evidenze/0.3.0/build.json; 21 Rust tests, 35 Windows self-test checks',
          'unchangedProductContract':['Four language sets, stable deduplication','Letter + Space; release letter confirms; Esc cancels',
             'Delay 100-1500 ms; theme system/light/dark; transparency preference','Explicit save, process-name exclusions, tray commands'],
          'prototype':'Four-section navigation; appearance links; error -> corrected draft -> saved demonstration.',
          'limitations':'Runtime verification is documented separately. The prototype does not execute hooks or persistence. Narrator, external apps and physical multi-monitor tests remain open.'}
        (OUT/'handoff.json').write_text(json.dumps(mapping,ensure_ascii=False,indent=2),encoding='utf-8')
        from figcraft_verify import verify
        verify()
        print(json.dumps({k:v for k,v in manifest.items() if k not in ['components','boards','limitations']},ensure_ascii=False),flush=True)

    def runtime_evidence(self):
        folder=OUT.parents[1]/'docs'/'evidenze'/'0.3.0'
        report=folder/'build.json'
        if not report.exists():return
        data=json.loads(report.read_text(encoding='utf-8'))
        p=self.page('08 / Prodotto implementato')
        self.text(p,'Accento 0.3 / Catture reali su Windows',0,0,3200,28,'on-accent',True,h=52)
        self.text(p,'Eseguibile verificato · 21 test Rust · 35 controlli Windows · SHA256 '+data['sha256'],0,64,3200,13,'on-accent',h=32)
        self.text(p,'Le pagine 00–07 restano editabili. Qui sono incorporati i pixel effettivi del prodotto, inclusa la superficie con alpha.',0,108,3200,13,'on-accent',h=32)
        for i,entry in enumerate(data['images']):
            x,y=(i%3)*1100,220+(i//3)*860
            self.text(p,entry['file'],x,y-36,1050,12,'on-accent',h=24)
            imported=self.m.command('file.import',path=str(folder/entry['file']),x=x,y=y)
            self.set(imported['id'],name='Runtime / '+entry['file'])
            mode=entry['mode']
            self.export_board(imported['id'],'runtime-'+entry['file'][:-4],mode,entry['width'],entry['height'],'runtime-capture')

def build():
    s=System()
    try:
        s.page('00 / Prodotto finale'); s.library(); s.popup_library(); s.settings_screens(); s.states()
        s.product(); s.foundations(); s.handoff(); s.tray(); s.runtime_evidence()
        order=['00 / Prodotto finale','01 / Popup trasparente','02 / Impostazioni finali','03 / Stati e feedback',
               '04 / Componenti','05 / Materiale e token','06 / Interazione e consegna','07 / Area di notifica']
        if len(s.pages)>8:order.append('08 / Prodotto implementato')
        pages=s.m.call('inspect_document')['pages']
        for i,name in enumerate(order):s.m.command('page.move',id=next(p['id'] for p in pages if p['name']==name),index=i)
        s.finish()
    finally:s.m.close()

if __name__=='__main__':build()
