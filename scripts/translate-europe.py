"""Create explicit draft dictionaries; never called by a normal/offline build.

Only generic product strings are sent to Google's public translation endpoint.
Existing dictionaries are preserved; human linguistic review remains required.
"""
import concurrent.futures
import json
import re
import time
import urllib.parse
import urllib.request
from pathlib import Path

root = Path(__file__).resolve().parent.parent
languages = json.loads((root/'localization/languages.json').read_text(encoding='utf8'))
source = (root/'crates/accento-windows/src/i18n.rs').read_text(encoding='utf8')
base_codes = ['it','fr','es','de','en','sq']
original = {code:{} for code in base_codes}
for key,values in re.findall(r'^\s+(\w+) => (\["[^\n]+\]),$',source,re.M):
    for code,value in zip(base_codes,json.loads(values)):
        original[code][key] = value
english = original['en'].copy()
english.update({
    'GuideTitle':'Accento quick start',
    'GuideStart':'Open Accento.exe on Windows. The settings interface is always dark.',
    'GuideWrite':'Hold a letter, press and release Space, then release the letter to insert the selected character. Use Left, Right or Space to change it. Escape cancels.',
    'GuideSets':'Open Character sets and enable the languages you write in. Keep at least one set active. Shared characters appear only once.',
    'GuideLanguage':'Choose the interface language in General. This does not change your active character sets.',
    'GuideSave':'Save changes applies your settings. Discard changes restores the last saved configuration. Pause takes effect immediately.',
    'GuideExit':'Closing settings leaves Accento running in the notification area. Choose Exit from its menu to stop it.',
    'GuideWindows':'Microsoft Edge WebView2 Runtime is required. The interface works offline. The input engine currently supports Windows only.',
    'GuideAlphabet':'The standard alphabet and available key associations are listed below. Type ordinary letters normally. The popup also supports digraphs and combining marks; it does not replace an input method.',
    'GuideReview':'This translated guide is a draft and has not been reviewed by a native speaker.',
})
path = root/'localization/messages.json'
progress = root/'localization/translation-progress.json'
existing = json.loads((path if path.exists() else progress).read_text(encoding='utf8')) if (path.exists() or progress.exists()) else {}

def translate_chunk(keys,target):
    single = len(keys) == 1
    text = english[keys[0]] if single else '\n'.join(f'[[{i:03d}]]: {english[key]}' for i,key in enumerate(keys))
    for token,replacement in [('Accento.exe','ZXQEXEQXZ'),('Accento','ZXQBRANDQXZ'),('notepad.exe','ZXQAPPQXZ')]:
        text = text.replace(token,replacement)
    url='https://translate.googleapis.com/translate_a/single?'+urllib.parse.urlencode({'client':'gtx','sl':'en','tl':target,'dt':'t','q':text})
    for attempt in range(4):
        try:
            data=json.loads(urllib.request.urlopen(url,timeout=30).read())
            translated=''.join(segment[0] or '' for segment in data[0])
            for token,replacement in [('ZXQEXEQXZ','Accento.exe'),('ZXQBRANDQXZ','Accento'),('ZXQAPPQXZ','notepad.exe'),('ЗКСКЕКСЕККСЗ','Accento.exe'),('ЗКСКБРАНДККСЗ','Accento'),('ЗКСКАППККСЗ','notepad.exe')]:
                translated=translated.replace(token,replacement)
            if single:return {keys[0]:translated.strip()}
            parts = re.split(r'\[{1,2}(\d{3})\]{1,2}\s*[:：.՝\-]?\s*',translated)
            values={int(parts[i]):parts[i+1].strip() for i in range(1,len(parts),2)}
            if set(values)!=set(range(len(keys))):raise ValueError(f'Unstable translation boundaries for {target}: {translated[:200]}')
            return {key:values[i] for i,key in enumerate(keys)}
        except Exception:
            if attempt==3:raise
            time.sleep(1+attempt)

# Nynorsk and Romansh drafts are explicit, not an English fallback or a mislabeled Bokmål translation.
manual={
'nn':'''KeepEditing|Hald fram med å redigere
Settings|Innstillingar
General|Generelt
Sets|Teiknsett
Blacklist|Utelatne program
GeneralHint|Skriving og språk. Alt på éin stad.
SetsHint|Vel teikna du vil ha lett tilgjengelege.
BlacklistHint|Program der Accento er slått av.
Delay|Opningsforseinking
DelayHint|Frå 100 til 1500 ms
Less|Kortare forseinking
More|Lengre forseinking
ActiveSets|Aktive teiknsett
SetDetail|Sjå alle teikna →
Appearance|Utsjånad
System|System
Light|Lys
Dark|Mørk
UiLanguage|Språk i grensesnittet
Independent|Uavhengig av aktive teiknsett.
Gesture|Bokstav + mellomrom: det første teiknet er allereie valt.
Release|← → eller mellomrom for å byte · Slepp bokstaven for å setje inn · Esc avbryt
SetsNote|Du kan slå på fleire sett. Felles teikn vert berre viste éin gong.
AtLeastOne|Hald minst eitt teiknsett aktivt.
AddApp|Legg til program
Add|Legg til
AppHint|Eitt namn om gongen, til dømes notepad.exe
AppPlaceholder|program.exe
Empty|Ingen utelatne program
EmptyHint|Accento er tilgjengeleg i alle program.
Remove|Fjern
Save|Lagre endringar
Saved|Alle endringar er lagra
CheckField|Kontroller det merkte feltet
Pending|Ulagra endringar
Enabled|Accento er på
Disabled|Accento er av
TurnOn|Slå på Accento
TurnOff|Slå av Accento
Pause|Pause
Resume|Hald fram
Exit|Avslutt
Selected|valt
NotSelected|ikkje valt
DelayError|Skriv inn ei forseinking mellom 100 og 1500 ms.
AppError|Skriv inn eitt .exe-namn, utan filsti eller komma.
Duplicate|Dette programmet er allereie utelate.
PickerError|Teiknveljaren er ikkje tilgjengeleg. Accento er av.
ConfigError|Konfigurasjonen er ikkje støtta.
LanguageError|Vel eit tilgjengeleg språk.
Variants|Variantar
InsertHint|Slepp bokstaven for å setje inn
Discard|Forkast endringar
SearchLanguage|Søk etter språk
NoLanguage|Fann ingen språk
ClearSearch|Tøm søket
Loading|Lastar…
Retry|Prøv igjen
Writing|Skriving
QuickGuide|Slik brukar du det
CloseDraft|Du har ulagra endringar. Vil du lukke og forkaste dei?
GuideTitle|Kom i gang med Accento
GuideStart|Opne Accento.exe i Windows. Innstillingane har alltid mørk utsjånad.
GuideWrite|Hald ein bokstav nede, trykk og slepp mellomrom, og slepp så bokstaven for å setje inn det valde teiknet. Bruk venstre, høgre eller mellomrom for å byte. Escape avbryt.
GuideSets|Opne Teiknsett og slå på språka du skriv på. Hald minst eitt sett aktivt. Felles teikn vert berre viste éin gong.
GuideLanguage|Vel språket for grensesnittet under Generelt. Dette endrar ikkje dei aktive teiknsetta.
GuideSave|Lagre endringar tek i bruk innstillingane. Forkast endringar hentar tilbake den sist lagra konfigurasjonen. Pause verkar med ein gong.
GuideExit|Når du lukkar innstillingane, køyrer Accento vidare i varslingsområdet. Vel Avslutt i menyen for å stoppe programmet.
GuideWindows|Microsoft Edge WebView2 Runtime er nødvendig. Grensesnittet verkar utan nett. Tastaturmotoren støttar førebels berre Windows.
GuideAlphabet|Standardalfabetet og tilgjengelege tastetilordningar står nedanfor. Skriv vanlege bokstavar som normalt. Veljaren støttar òg digrafar og kombinerande teikn, men erstattar ikkje ein inndatametode.
GuideReview|Denne omsette rettleiinga er eit utkast og er ikkje gjennomgått av ein morsmålsbrukar.''',
'rm':'''KeepEditing|Cuntinuar a modifitgar
Settings|Parameters
General|General
Sets|Gieus da caracters
Blacklist|Applicaziuns exclusas
GeneralHint|Scriver e lingua. Tut en in lieu.
SetsHint|Tscherna ils caracters che ti vuls avair a disposiziun.
BlacklistHint|Applicaziuns nua che Accento resta deactivà.
Delay|Retard d'avertura
DelayHint|Da 100 fin 1500 ms
Less|Reducir il retard
More|Augmentar il retard
ActiveSets|Gieus da caracters activs
SetDetail|Vesair tut ils caracters →
Appearance|Apparientscha
System|Sistem
Light|Cler
Dark|Stgir
UiLanguage|Lingua da l'interfatscha
Independent|Independent dals gieus da caracters activs.
Gesture|Bustab + spazi: l'emprim caracter è gia tschernì.
Release|← → u spazi per midar · Lascha liber il bustab per inserir · Esc interrumpa
SetsNote|Ti pos activar plirs gieus. Caracters communabels cumparan mo ina giada.
AtLeastOne|Mantegna almain in gieu da caracters activ.
AddApp|Agiuntar ina applicaziun
Add|Agiuntar
AppHint|In num a la giada, per exempel notepad.exe
AppPlaceholder|applicaziun.exe
Empty|Naginas applicaziuns exclusas
EmptyHint|Accento è disponibel en tut las applicaziuns.
Remove|Allontanar
Save|Memorisar las midadas
Saved|Tut las midadas èn memorisadas
CheckField|Controlla il champ marcà
Pending|Midadas betg memorisadas
Enabled|Accento è activ
Disabled|Accento è deactivà
TurnOn|Activar Accento
TurnOff|Deactivar Accento
Pause|Pausa
Resume|Cuntinuar
Exit|Serrar
Selected|tschernì
NotSelected|betg tschernì
DelayError|Endatescha in retard tranter 100 e 1500 ms.
AppError|Endatescha in sulet num .exe, senza percurs u commas.
Duplicate|Questa applicaziun è gia exclusa.
PickerError|Il selectur da caracters n'è betg disponibel. Accento è deactivà.
ConfigError|Configuraziun betg sustegnida.
LanguageError|Tscherna ina lingua disponibla.
Variants|Variantas
InsertHint|Lascha liber il bustab per inserir
Discard|Sbittar las midadas
SearchLanguage|Tschertgar ina lingua
NoLanguage|Naginas linguas chattadas
ClearSearch|Stizzar la tschertga
Loading|Chargiar…
Retry|Empruvar anc ina giada
Writing|Scriver
QuickGuide|Co funcziuna quai
CloseDraft|Ti has midadas betg memorisadas. Serrar e las sbittar?
GuideTitle|Introducziun rapida ad Accento
GuideStart|Avra Accento.exe en Windows. L'interfatscha dals parameters è adina stgira.
GuideWrite|Tegna in bustab, smatga e lascha liber il spazi, lura lascha liber il bustab per inserir il caracter tschernì. Dovra las frizzas u il spazi per midar. Escape interrumpa.
GuideSets|Avra Gieus da caracters ed activa las linguas che ti scrivas. Mantegna almain in gieu activ. Caracters communabels cumparan mo ina giada.
GuideLanguage|Tscherna la lingua da l'interfatscha sut General. Quai na mida betg ils gieus da caracters activs.
GuideSave|Memorisar las midadas applitgescha ils parameters. Sbittar las midadas restaurescha l'ultima configuraziun memorisada. Pausa ha effect immediat.
GuideExit|Sche ti serras ils parameters, resta Accento activ en la zona da notificaziun. Tscherna Serrar en il menu per fermar il program.
GuideWindows|Microsoft Edge WebView2 Runtime è necessari. L'interfatscha funcziuna senza internet. Il motor da tastatura sustegna actualmain mo Windows.
GuideAlphabet|L'alfabet standard e las associaziuns da tastas èn inditgads qua sut. Scriva normalmain ils bustabs ordinairs. Il selectur sustegna era digrafs e segns cumbinants, ma na remplazza betg ina metoda d'endataziun.
GuideReview|Questa guida translatada è in sboz e n'è betg vegnida verifitgada d'ina persuna da lingua materna.'''
}
manual={code:dict(row.split('|',1) for row in text.splitlines()) for code,text in manual.items()}
CYR='абвгдђежзијклљмнњопрстћуфхцчџш'
LAT=['a','b','v','g','d','đ','e','ž','z','i','j','k','l','lj','m','n','nj','o','p','r','s','t','ć','u','f','h','c','č','dž','š']
def latinize(value):
    mapping=dict(zip(CYR,LAT));mapping.update({c.upper():v.title() for c,v in zip(CYR,LAT)})
    return ''.join(mapping.get(c,c) for c in value)

def work(language):
    code=language['code']
    if code in manual:return code,manual[code]
    if code in ('bs-Cyrl','cnr-Cyrl'):return code,existing[code]
    dictionary={**existing.get(code,{}),**original.get(code,{})}
    if code not in original: dictionary.pop('GeneralHint',None)
    dictionary.pop('GuideAlphabet',None)
    missing=[key for key in english if not dictionary.get(key)]
    target={'nb':'no','sr-Latn':'sr','cnr':'sr'}.get(code,code)
    for offset in range(0,len(missing),18):
        dictionary.update(translate_chunk(missing[offset:offset+18],target))
    if code in ('sr-Latn','cnr'):dictionary={k:latinize(v) for k,v in dictionary.items()}
    assert set(dictionary)==set(english),(code,set(dictionary)^set(english))
    return code,dictionary

result={};failures=[]
with concurrent.futures.ThreadPoolExecutor(max_workers=3) as pool:
    futures={pool.submit(work,language):language['code'] for language in languages}
    for future in concurrent.futures.as_completed(futures):
        try:
            code,dictionary=future.result();result[code]=dictionary
            print(code,'ready',flush=True)
            # Preserve completed drafts if a provider rejects one language or the run is interrupted.
            (root/'localization/translation-progress.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
        except Exception as error:failures.append((futures[future],str(error)))
if failures:raise RuntimeError(failures)
cyr_pairs=list(zip(['a','b','v','g','d','đ','e','ž','z','i','j','k','l','lj','m','n','nj','o','p','r','s','t','ć','u','f','h','c','č','dž','š','ś','ź'],list('абвгдђежзијклљмнњопрстћуфхцчџш')+['с́','з́']))
cyr_map=dict(cyr_pairs);cyr_map.update({a.title():b.upper() for a,b in cyr_pairs});cyr_map.update({a.upper():b.upper() for a,b in cyr_pairs})
cyr_pattern=re.compile('|'.join(re.escape(k) for k in sorted(cyr_map,key=len,reverse=True)))
def cyrillicize(value):
    chunks=re.split(r'(Accento(?:\.exe)?|notepad\.exe|Microsoft Edge WebView2 Runtime|Windows|Unicode|Space|Escape|Esc)',value)
    return ''.join(chunk if i%2 else cyr_pattern.sub(lambda m:cyr_map[m[0]],chunk) for i,chunk in enumerate(chunks))
for code,base in [('bs-Cyrl','bs'),('cnr-Cyrl','cnr')]:result[code]={key:cyrillicize(value) for key,value in result[base].items()}
result={language['code']:result[language['code']] for language in languages}
for code,values in json.loads((root/'localization/overrides.json').read_text(encoding='utf8')).items():result[code].update(values)
path.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
(root/'localization/translation-status.json').write_text(json.dumps({language['code']:{'uiSource':'existing' if language['code'] in original else 'assistant-draft' if language['code'] in manual else 'Google machine translation draft','nativeReview':False,'guideSource':'assistant-draft' if language['code'] in manual else 'Google machine translation draft'} for language in languages},ensure_ascii=False,indent=2)+'\n',encoding='utf8')
print('Complete explicit dictionaries written; no missing-language English fallback.')
