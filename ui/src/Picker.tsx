import React,{useEffect,useState} from 'react';
import {invoke} from '@tauri-apps/api/core';
import {listen} from '@tauri-apps/api/event';
type View={candidates:string[];selected:number|null;generation:number};
export function Picker(){
  const [view,setView]=useState<View|null>(null);
  const [width,setWidth]=useState(innerWidth);
  useEffect(()=>{document.documentElement.classList.add('picker-surface');let disposed=false;let stop:undefined|(()=>void);
    void listen<View|null>('accento-picker',e=>{if(!disposed)setView(e.payload);}).then(unlisten=>{if(disposed)unlisten();else{stop=unlisten;void invoke<View|null>('get_picker').then(value=>{if(!disposed)setView(value);});}});
    const resize=()=>setWidth(innerWidth);addEventListener('resize',resize);
    return()=>{disposed=true;stop?.();removeEventListener('resize',resize);};},[]);
  if(!view)return null;
  const count=Math.max(1,Math.floor((width-8)/72));
  const start=Math.max(0,Math.min((view.selected??0)-Math.floor(count/2),view.candidates.length-count));
  return <div className="native-picker" role="listbox" aria-label="Accento" aria-live="polite">{view.candidates.slice(start,start+count).map((text,index)=><div key={start+index} role="option" aria-selected={view.selected===start+index} className={'native-key '+(view.selected===start+index?'active':'')}><span>{text}</span></div>)}</div>;
}
