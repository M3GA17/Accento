import React, {useCallback, useEffect, useRef, useState} from 'react';
import {Button, TextBox, LanguageSelector} from '@dimalabs/react-component-flat-product';
import {Settings2, Languages, AppWindow, Check, Plus, X, Keyboard, Globe2} from 'lucide-react';
import * as api from './api';
import {interfaceLanguages,characterSets} from './catalog';
import dictionaries from './messages.json';
import {version} from '../../package.json';

type Locale = keyof typeof dictionaries;
type Message = keyof typeof dictionaries.it;
type Section = 'General'|'Sets'|'Blacklist';
const sections = [{key:'General',icon:Settings2},{key:'Sets',icon:Languages},{key:'Blacklist',icon:AppWindow}] as const;
const equal = (a:api.Preferences,b:api.Preferences)=>JSON.stringify(a)===JSON.stringify(b);
const message = (error:unknown)=>error instanceof Error ? error.message : String(error);

function BrandMark() {
  return <img className="brand-mark" src="/brand/accento.svg" alt="" aria-hidden="true" width={44} height={44}/>;
}

export function App() {
  const [saved,setSaved] = useState<api.Preferences>();
  const [draft,setDraft] = useState(api.defaults);
  const [delay,setDelay] = useState('300');
  const [section,setSection] = useState<Section>('General');
  const [setQuery,setSetQuery] = useState('');
  const [paused,setPaused] = useState(false);
  const [platform,setPlatform] = useState<api.PlatformInfo>(api.previewPlatform);
  const [warning,setWarning] = useState<string|null>(null);
  const [error,setError] = useState('');
  const [delayError,setDelayError] = useState(false);
  const [entry,setEntry] = useState('');
  const [entryError,setEntryError] = useState<Message>();
  const [busy,setBusy] = useState(false);
  const [loading,setLoading] = useState(true);
  const [closing,setClosing] = useState(false);
  const [quitting,setQuitting] = useState(false);
  const closeAllowed = useRef(false);
  const closeDialog = useRef<HTMLDialogElement>(null);
  useEffect(()=>{if(closing)closeDialog.current?.showModal();},[closing]);
  const dirty = !!saved && (!equal(draft,saved) || delay!==String(saved.delay_ms));
  const dirtyRef = useRef(dirty);dirtyRef.current=dirty;
  const locale = (draft.ui_language in dictionaries ? draft.ui_language : 'it') as Locale;
  const t = (key:Message)=>dictionaries[locale][key];
  useEffect(()=>{
    document.documentElement.dataset.dimalabsTheme='dark';
    document.documentElement.lang=locale;document.title=`Accento · ${t('Settings')}`;
    document.documentElement.style.colorScheme='dark';
  },[locale]);
  const accept = useCallback((snapshot:api.Snapshot,reset=false)=>{
    setSaved(snapshot.preferences);setPaused(snapshot.paused);setWarning(snapshot.warning);
    if(reset || !dirtyRef.current) {setDraft(snapshot.preferences);setDelay(String(snapshot.preferences.delay_ms));}
  },[]);
  const load = useCallback(async()=>{
    setLoading(true);setError('');
    try {const [snapshot,platform]=await Promise.all([api.read(),api.platform()]);setPlatform(platform);accept(snapshot,true);}catch(error){setError(message(error));}
    finally{setLoading(false);}
  },[accept]);
  useEffect(()=>{
    let disposed=false;let stop:undefined|(()=>void);
    // Subscribe before reading so changes from the tray cannot slip between read and subscription.
    void api.subscribe(snapshot=>{if(!disposed) accept(snapshot);}).then(unlisten=>{
      if(disposed) unlisten();else{stop=unlisten;void load();}
    }).catch(error=>{if(!disposed){setError(message(error));setLoading(false);}});
    return ()=>{disposed=true;stop?.();};
  },[accept,load]);
  useEffect(()=>{
    if(!api.desktop)return;
    let disposed=false;let stop:undefined|(()=>void);
    void import('@tauri-apps/api/window').then(({getCurrentWindow})=>getCurrentWindow().onCloseRequested(event=>{
      if(dirtyRef.current && !closeAllowed.current){event.preventDefault();setQuitting(false);setClosing(true);}
    })).then(unlisten=>{if(disposed)unlisten();else stop=unlisten;}).catch(error=>setError(message(error)));
    return ()=>{disposed=true;stop?.();};
  },[]);
  function update(change:Partial<api.Preferences>){setDraft(current=>({...current,...change}));setError('');}
  function toggleSet(code:string){
    const selected=draft.languages.includes(code);
    if(selected && draft.languages.length===1){setError(t('AtLeastOne'));return;}
    update({languages:selected?draft.languages.filter(item=>item!==code):[...draft.languages,code]});
  }
  function addApp(event:React.FormEvent){
    event.preventDefault();const name=entry.trim().toLowerCase();
    if(!name || [".","..",".exe",".app"].includes(name) || (platform.os==='windows' && (!name.endsWith('.exe') || name.length<=4)) || /[\\/:*?"<>|,;\x00-\x1f\x7f]/.test(name)){setEntryError(platform.os==='windows'?'AppError':'NativeAppError');return;}
    if(draft.excluded_apps.some(app=>app.toLowerCase()===name)){setEntryError('Duplicate');return;}
    update({excluded_apps:[...draft.excluded_apps,name]});setEntry('');setEntryError(undefined);
  }
  async function save(){
    if(!/^\d+$/.test(delay) || Number(delay)<100 || Number(delay)>1500){setDelayError(true);setSection('General');return;}
    setBusy(true);setError('');
    try{accept(await api.save({...draft,delay_ms:Number(delay)}),true);setDelayError(false);}
    catch(error){setError(message(error));}finally{setBusy(false);}
  }
  async function togglePause(){
    setBusy(true);setError('');
    try{accept(await api.pause(!paused));}catch(error){setError(message(error));}finally{setBusy(false);}
  }
  function discard(){if(saved){setDraft(saved);setDelay(String(saved.delay_ms));setError('');setDelayError(false);setEntryError(undefined);}}
  async function close(){
    closeAllowed.current=true;setClosing(false);
    try{if(quitting)await api.quit();else{const {getCurrentWindow}=await import('@tauri-apps/api/window');await getCurrentWindow().close();}}
    catch(error){closeAllowed.current=false;setError(message(error));}
  }
  if(!saved) return <div className="startup"><BrandMark/><h1>Accento</h1>
    {loading?<p role="status">{t('Loading')}</p>:<><p role="alert">{error}</p><Button onPress={()=>void load()}>{t('Retry')}</Button></>}
  </div>;
  return <div className="shell">
    <aside className="sidebar">
      <div className="brand"><BrandMark/><div><strong>Accento</strong><span>{t('Settings')}</span></div></div>
      <nav aria-label={t('Settings')}>{sections.map(({key,icon:Icon})=><Button key={key} variant="quiet" className={'nav-item '+(section===key?'selected':'')}
        aria-current={section===key?'page':undefined} onPress={()=>setSection(key)}><Icon size={19}/>{t(key)}</Button>)}</nav>
      <div className="sidebar-bottom"><div className="runtime"><span className={'status-dot '+(paused?'off':'')}/><span>{t(paused?'Disabled':'Enabled')}</span></div>
        <Button variant="secondary" disabled={busy} onPress={()=>void togglePause()}>{t(paused?'TurnOn':'TurnOff')}</Button>
        {api.desktop&&<Button className="exit-app" variant="quiet" onPress={()=>{if(dirty){setQuitting(true);setClosing(true);}else void api.quit().catch(error=>setError(message(error)));}}>{t('Exit')}</Button>}
        <span className="version">Accento {version}</span>
      </div>
    </aside>
    <div className="workspace">
      {api.preview && <div className="preview-banner">Anteprima · dati temporanei</div>}
      <main id="main-content" key={section}>
        <header className="page-heading"><span className="eyebrow">ACCENTO / {t('Settings')}</span><h1>{t(section)}</h1>
          <p>{t(section==='General'?'GeneralHint':section==='Sets'?'SetsHint':'BlacklistHint')}</p></header>
        {warning && <div className="notice" role="alert">{warning}</div>}
        {error && <div className="notice error" role="alert">{error}</div>}
        <fieldset className="settings-fields" disabled={busy}>
        {section==='General' && <>
          <section className="settings-group"><h2><Keyboard size={18}/>{t('Writing')}</h2>
            <div className="setting-row"><div><h3>{t('Delay')}</h3><p>{t('DelayHint')}</p></div>
              <div className="delay-control"><TextBox aria-label={t('Delay')} value={delay} inputMode="numeric" maxLength={4}
                onValueChange={value=>{setDelay(value);setDelayError(false);}} error={delayError?t('DelayError'):undefined}/><span>ms</span></div></div>
          </section>
          <section className="settings-group"><div className="setting-row language-row"><div><h2><Globe2 size={18}/>{t('UiLanguage')}</h2><p>{t('Independent')}</p></div>
            <LanguageSelector label={t('UiLanguage')} options={interfaceLanguages} value={draft.ui_language} display="name" searchable
              onValueChange={ui_language=>update({ui_language})} searchLabel={t('SearchLanguage')} searchPlaceholder={t('SearchLanguage')}
              clearSearchLabel={t('ClearSearch')} emptyMessage={t('NoLanguage')} loadingLabel={t('Loading')}/></div></section>
          <div className="guide"><span className="keycap">e</span><span>+</span><span className="keycap">Space</span><div><strong>{t('QuickGuide')}</strong><p>{t('Gesture')}</p></div></div>
        </>}
        {section==='Sets' && <><p className="section-note">{t('SetsNote')}</p>
          <TextBox label={t('SearchLanguage')} value={setQuery} onValueChange={setSetQuery} className="set-search"/>
          <div className="sets-grid">{characterSets.filter(set=>[set.name,set.english,set.code,...set.countries].join(' ').normalize('NFD').replace(/[\u0300-\u036f]/g,'').toLowerCase().includes(setQuery.normalize('NFD').replace(/[\u0300-\u036f]/g,'').toLowerCase().trim())).map(set=>
          <Button key={set.code} data-language={set.code} variant="quiet" className={'set-card '+(draft.languages.includes(set.code)?'chosen':'')} aria-pressed={draft.languages.includes(set.code)} onPress={()=>toggleSet(set.code)}>
            <span className="set-title"><span>{set.name}</span><span className="selection-box">{draft.languages.includes(set.code)&&<Check size={14}/>}</span></span>
            <span className="set-sample">{set.sample}</span>
          </Button>)}</div></>}
        {section==='Blacklist' && <>
          <form className="add-app" onSubmit={addApp}><TextBox label={t('AddApp')} description={t(platform.os==='windows'?'AppHint':'NativeAppHint')} placeholder={platform.executable_example} value={entry}
            onValueChange={value=>{setEntry(value);setEntryError(undefined);}} error={entryError?t(entryError):undefined} maxLength={260}/>
            <Button type="submit" disabled={!entry.trim()}><Plus size={16}/>{t('Add')}</Button></form>
          {draft.excluded_apps.length?<ul className="app-list">{draft.excluded_apps.map(name=><li key={name}><AppWindow size={20}/><span>{name}</span>
            <Button variant="quiet" aria-label={`${t('Remove')} ${name}`} onPress={()=>update({excluded_apps:draft.excluded_apps.filter(app=>app!==name)})}><X size={18}/></Button></li>)}</ul>
            :<div className="empty-state"><AppWindow size={32}/><h3>{t('Empty')}</h3><p>{t('EmptyHint')}</p></div>}
        </>}
        </fieldset>
      </main>
      <footer><span className="save-status" role="status">{dirty?<span className="pending-dot"/>:<Check size={16}/>}<span>{t(dirty?'Pending':'Saved')}</span></span>
        <div>{dirty&&<Button variant="quiet" disabled={busy} onPress={discard}>{t('Discard')}</Button>}<Button variant="primary" disabled={!dirty||busy} loading={busy} onPress={()=>void save()}>{t('Save')}</Button></div>
      </footer>
    </div>
    {closing&&<dialog ref={closeDialog} role="alertdialog" aria-labelledby="close-question" className="close-dialog" onCancel={()=>setClosing(false)}><h2 id="close-question">{t('CloseDraft')}</h2><div>
      <Button autoFocus onPress={()=>setClosing(false)}>{t('KeepEditing')}</Button><Button variant="danger" onPress={()=>void close()}>{t('Discard')}</Button>
    </div></dialog>}
  </div>;
}
