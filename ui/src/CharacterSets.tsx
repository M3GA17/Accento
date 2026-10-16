import {useEffect,useRef,useState} from 'react';
import {Check,ChevronDown} from 'lucide-react';
import {Button,TextBox} from './components';
import {characterSets} from './catalog';
import type {Preferences} from './api';
import type dictionaries from './messages.json';
import currencies from '../../localization/currencies.json';

type Props = {
  draft:Preferences;
  update:(change:Partial<Preferences>)=>void;
  onError:(message:string)=>void;
  query:string;
  onQueryChange:(value:string)=>void;
  text:typeof dictionaries.it;
};
const normalize=(value:string)=>value.normalize('NFD').replace(/[\u0300-\u036f]/g,'').toLowerCase();

export function CharacterSets({draft,update,onError,query,onQueryChange,text}:Props) {
  const [expanded,setExpanded]=useState(true);
  const allRef=useRef<HTMLInputElement>(null);
  const allSelected=characterSets.every(set=>draft.languages.includes(set.code))&&draft.currency_symbols;
  useEffect(()=>{if(allRef.current)allRef.current.indeterminate=!allSelected;},[allSelected]);
  const filtered=characterSets.filter(set=>normalize([set.name,set.english,set.code,...set.countries].join(' ')).includes(normalize(query.trim())));
  function toggleSet(code:string) {
    const selected=draft.languages.includes(code);
    if(selected&&draft.languages.length===1){onError(text.AtLeastOne);return;}
    update({languages:selected?draft.languages.filter(item=>item!==code):[...draft.languages,code]});
  }
  function toggleAll() {
    if(allSelected){
      // Preserve the first active language: preferences always require one.
      update({languages:draft.languages.slice(0,1),currency_symbols:false});
      onError(text.AtLeastOne);
    }else update({languages:characterSets.map(set=>set.code),currency_symbols:true});
  }
  return <section className="sets-panel" aria-label={text.Sets}>
    <div className="sets-panel-header">
      <Button variant="quiet" className="sets-disclosure" aria-expanded={expanded} aria-controls="sets-panel-content" onPress={()=>setExpanded(value=>!value)}>
        <span className="sets-panel-copy"><strong>{text.Sets}</strong></span>
        <ChevronDown size={17} className={expanded?'expanded':''}/>
      </Button>
      <label className="set-option select-all"><input ref={allRef} type="checkbox" checked={allSelected} onChange={toggleAll}/><span className="set-checkbox" aria-hidden="true">{allSelected?<Check size={13}/>:<span className="mixed-mark"/>}</span><span>{text.AllAvailable}</span></label>
    </div>
    <div id="sets-panel-content" className="sets-panel-content" hidden={!expanded}>
      <section className="language-sets" aria-labelledby="language-sets-heading">
        <div className="set-group-heading"><h2 id="language-sets-heading">{text.LanguageSets}</h2>
          <TextBox aria-label={text.SearchLanguage} placeholder={text.SearchLanguage} value={query} onValueChange={onQueryChange} className="set-search"/>
        </div>
        <div className="sets-grid">{filtered.map(set=><label key={set.code} className="set-option" title={set.sample}>
          <input type="checkbox" data-language={set.code} checked={draft.languages.includes(set.code)} aria-description={set.sample} onChange={()=>toggleSet(set.code)}/>
          <span className="set-checkbox" aria-hidden="true"><Check size={13}/></span><span>{set.name}</span>
        </label>)}</div>
        {!filtered.length&&<p className="set-search-empty" role="status">{text.NoLanguage}</p>}
      </section>
      <section className="special-sets" aria-labelledby="special-sets-heading">
        <div className="set-group-heading"><h2 id="special-sets-heading">{text.SpecialSets}</h2></div>
        <div className="sets-grid"><label className="set-option" title="e → € · s → $ · l → £ · y → ¥">
          <input type="checkbox" data-special-set="currency" checked={draft.currency_symbols} onChange={()=>update({currency_symbols:!draft.currency_symbols})}/>
          <span className="set-checkbox" aria-hidden="true"><Check size={13}/></span><span>{text.Currency}</span>
        </label></div>
        <details className="currency-details"><summary>{text.Currency} · {text.Variants}</summary>
          <dl>{Object.entries(currencies).map(([key,symbols])=><div key={key}><dt><kbd>{key}</kbd></dt><dd>{symbols.join(' · ')}</dd></div>)}</dl>
        </details>
      </section>
    </div>
  </section>;
}
