"""Refresh pinned CLDR exemplars and derived character maps (not run during builds)."""
import concurrent.futures
import json
import re
import unicodedata as ud
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DATA = ROOT / 'localization'
DATA.mkdir(exist_ok=True)
ROWS = '''it|Italiano|Italian|Latin|IT SM VA|Italian
en|English|English|Latin|GB IE MT|English
sq|Shqip|Albanian|Latin|AL XK|Albanian
fr|Français|French|Latin|FR BE CH LU MC|French
es|Español|Spanish|Latin|ES|Spanish
de|Deutsch|German|Latin|DE AT BE CH LI LU|German
pt|Português|Portuguese|Latin|PT|Portuguese
ro|Română|Romanian|Latin|RO MD|Romanian
pl|Polski|Polish|Latin|PL|Polish
tr|Türkçe|Turkish|Latin|TR CY|Turkish
nl|Nederlands|Dutch|Latin|NL BE|Dutch
lb|Lëtzebuergesch|Luxembourgish|Latin|LU|Luxembourgish
rm|Rumantsch|Romansh|Latin|CH|Romansh
da|Dansk|Danish|Latin|DK|Danish
sv|Svenska|Swedish|Latin|SE FI|Swedish
nb|Norsk bokmål|Norwegian Bokmål|Latin|NO|NorwegianBokmal
nn|Norsk nynorsk|Norwegian Nynorsk|Latin|NO|NorwegianNynorsk
fi|Suomi|Finnish|Latin|FI|Finnish
is|Íslenska|Icelandic|Latin|IS|Icelandic
et|Eesti|Estonian|Latin|EE|Estonian
lv|Latviešu|Latvian|Latin|LV|Latvian
lt|Lietuvių|Lithuanian|Latin|LT|Lithuanian
cs|Čeština|Czech|Latin|CZ|Czech
sk|Slovenčina|Slovak|Latin|SK|Slovak
hu|Magyar|Hungarian|Latin|HU|Hungarian
sl|Slovenščina|Slovenian|Latin|SI|Slovenian
hr|Hrvatski|Croatian|Latin|HR BA|Croatian
bs|Bosanski|Bosnian|Latin|BA|Bosnian
bs-Cyrl|Босански (ћирилица)|Bosnian (Cyrillic)|Cyrillic|BA|BosnianCyrillic
sr|Српски|Serbian|Cyrillic|RS BA XK|Serbian
sr-Latn|Srpski (latinica)|Serbian (Latin)|Latin|RS BA XK|SerbianLatin
cnr|Crnogorski|Montenegrin|Latin|ME|Montenegrin
cnr-Cyrl|Црногорски (ћирилица)|Montenegrin (Cyrillic)|Cyrillic|ME|MontenegrinCyrillic
mk|Македонски|Macedonian|Cyrillic|MK|Macedonian
bg|Български|Bulgarian|Cyrillic|BG|Bulgarian
el|Ελληνικά|Greek|Greek|GR CY|Greek
be|Беларуская|Belarusian|Cyrillic|BY|Belarusian
uk|Українська|Ukrainian|Cyrillic|UA|Ukrainian
ru|Русский|Russian|Cyrillic|RU BY KZ|Russian
ga|Gaeilge|Irish|Latin|IE|Irish
mt|Malti|Maltese|Latin|MT|Maltese
ca|Català|Catalan|Latin|AD|Catalan
hy|Հայերեն|Armenian|Armenian|AM|Armenian
ka|ქართული|Georgian|Georgian|GE|Georgian
az|Azərbaycanca|Azerbaijani|Latin|AZ|Azerbaijani
kk|Қазақша|Kazakh|Cyrillic|KZ|Kazakh
la|Latina|Latin|Latin|VA|Latin'''
languages = []
for row in ROWS.splitlines():
    code, name, english, script, countries, variant = row.split('|')
    languages.append(dict(code=code, name=name, english=english, script=script, countries=countries.split(), variant=variant))
BASE = 'https://raw.githubusercontent.com/unicode-org/cldr-json/48.0.0/cldr-json/cldr-misc-full/main/'

def fetch(language):
    code = language['code']
    if code in ('cnr','cnr-Cyrl'):
        source = BASE + ('sr-Latn' if code == 'cnr' else 'sr') + '/characters.json'
        data = json.loads(urllib.request.urlopen(source, timeout=30).read())
        characters = next(iter(data['main'].values()))['characters'].copy()
        characters['exemplarCharacters'] = characters['exemplarCharacters'][:-1] + (' ś ź]' if code == 'cnr' else ' {с́} {з́}]')
        source += ' + Montenegrin Pravopis 2010: ś, ź'
    elif code == 'la':
        characters = {'exemplarCharacters': '[a b c d e f g h i j k l m n o p q r s t u v w x y z]', 'auxiliary': '[ā ē ī ō ū ȳ]'}
        source = 'Latin alphabet; optional vowel-length macrons are explicitly auxiliary'
    else:
        source = BASE + code + '/characters.json'
        data = json.loads(urllib.request.urlopen(source, timeout=30).read())
        characters = next(iter(data['main'].values()))['characters']
    return code, dict(source=source, main=characters['exemplarCharacters'], auxiliary=characters.get('auxiliary', '[]'))

def parse_set(value):
    tokens = re.findall(r'\{[^}]+\}|\\u[0-9a-fA-F]{4}|\\.|[^\s]', value[1:-1])
    result = []
    index = 0
    while index < len(tokens):
        token = tokens[index]
        if index + 2 < len(tokens) and tokens[index+1] == '-':
            result.extend(chr(n) for n in range(ord(token), ord(tokens[index+2])+1))
            index += 3
            continue
        if token.startswith('{'): token = token[1:-1]
        elif token.startswith('\\u'): token = chr(int(token[2:],16))
        elif token.startswith('\\'): token = token[1:]
        result.append(ud.normalize('NFC', token))
        index += 1
    return list(dict.fromkeys(result))

# Convenience key associations, not a transliteration engine. Native keys also work.
SPECIAL = {'ŧ':'t','ǥ':'g','ѣ':'e','ѫ':'u','ʼ':"'", 'æ':'a','œ':'o','ß':'s','ø':'o','ł':'l','đ':'d','ð':'d','þ':'t','ħ':'h','ı':'i','ŋ':'n','ĸ':'k','ə':'e','ʒ':'z'}
SPECIAL.update(dict(zip('αβγδεζηθικλμνξοπρσςτυφχψω', 'abgdezhtiklmnxoprsstyfcpo')))
SPECIAL.update({'а':'a','б':'b','в':'v','г':'g','д':'d','е':'e','ё':'e','ж':'z','з':'z','и':'i','й':'i','к':'k','л':'l','м':'m','н':'n','о':'o','п':'p','р':'r','с':'s','т':'t','у':'u','ф':'f','х':'h','ц':'c','ч':'c','ш':'s','щ':'s','ъ':'y','ы':'y','ь':'y','э':'e','ю':'u','я':'a','і':'i','ї':'i','є':'e','ґ':'g','ў':'u','ђ':'d','ј':'j','љ':'l','њ':'n','ћ':'c','џ':'d','ѓ':'g','ќ':'k','ѕ':'z','ә':'a','ғ':'g','қ':'k','ң':'n','ө':'o','ұ':'u','ү':'u','һ':'h'})
SPECIAL.update(dict(part.split(':') for part in 'ա:a բ:b գ:g դ:d ե:e զ:z է:e ը:e թ:t ժ:z ի:i լ:l խ:h ծ:c կ:k հ:h ձ:d ղ:g ճ:c մ:m յ:y ն:n շ:s ո:o չ:c պ:p ջ:j ռ:r ս:s վ:v տ:t ր:r ց:c ւ:u փ:p ք:k օ:o ֆ:f և:e'.split()))
SPECIAL.update(dict(part.split(':') for part in 'ა:a ბ:b გ:g დ:d ე:e ვ:v ზ:z თ:t ი:i კ:k ლ:l მ:m ნ:n ო:o პ:p ჟ:z რ:r ს:s ტ:t უ:u ფ:f ქ:k ღ:g ყ:q შ:s ჩ:c ც:c ძ:z წ:w ჭ:c ხ:h ჯ:j ჰ:h'.split()))

snapshot = DATA/'cldr48.json'
if snapshot.exists():
    exemplars = json.loads(snapshot.read_text(encoding='utf8'))
else:
    exemplars = {}

with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
    exemplars.update(dict(pool.map(fetch,[language for language in languages if language['code'] not in exemplars])))
snapshot.write_text(json.dumps(exemplars,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
output = []
missing = {}
for language in languages:
    code = language['code']
    entry = exemplars[code]
    main = parse_set(entry['main'])
    auxiliary = parse_set(entry['auxiliary'])
    maps = {}
    # Keep historical accents and common auxiliary single characters for continuity.
    for value in main + (auxiliary if code == 'la' else []):
        if not value or not value[0].isalpha() or any(not (c.isalpha() or ud.combining(c)) for c in value[1:]): continue
        decomposed = ''.join(c for c in ud.normalize('NFKD',value) if not ud.combining(c))
        native_base = decomposed[0].lower()
        latin_base = SPECIAL.get(native_base,native_base)
        if not latin_base.isascii() or not (latin_base.isalpha() or latin_base == "'"):
            # No silent loss: an unmapped alphabet character is a generation error.
            if not value.isascii():
                missing[value] = ud.name(value)
                continue
        upper = ud.normalize('NFC',value.upper())
        if value == 'ß': upper = 'ẞ'
        for base in dict.fromkeys([native_base,latin_base]):
            if value == base and len(decomposed)==1: continue
            maps.setdefault(base,[])
            pair = [value,upper]
            if pair not in maps[base]: maps[base].append(pair)
    if code == 'ca':
        maps.setdefault('l',[]).extend([['l·l','L·L'],['·','·']])
    if code == 'it':
        maps['e'] = [['è','È'],['é','É']]
        maps['o'] = [['ò','Ò'],['ó','Ó']]
    if code == 'ru':
        for vowel in 'аеиоуыэюя':
            stressed = vowel + '\u0301'
            for base in dict.fromkeys([vowel,SPECIAL[vowel]]):
                maps.setdefault(base,[]).append([stressed,stressed.upper()])
    # Turkish/Azerbaijani dotted and dotless I need locale-specific case mapping.
    if code in ('tr','az'):
        maps['i'] = [['ı','I'],['i','İ']]
    entry.update(mainCharacters=main, auxiliaryCharacters=auxiliary, maps=maps)
    sample = ' '.join(value for value in main if not (value.isascii() and len(value)==1))
    if not sample: sample = ' '.join(main)
    output.append({**language,'sample':sample,'alphabet':' '.join(main),'digraphs':[v for v in main if len(v)>1 and all(c.isalpha() for c in v)]})
if missing: raise ValueError(missing)
(DATA/'languages.json').write_text(json.dumps(output,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
(DATA/'characters.json').write_text(json.dumps(exemplars,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
print(f'{len(output)} locale repertoires imported from CLDR 48 (plus documented cnr/la supplements).')
