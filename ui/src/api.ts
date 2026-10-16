import {invoke, isTauri} from '@tauri-apps/api/core';
import {listen} from '@tauri-apps/api/event';
export type Preferences = {schema_version:number;delay_ms:number;languages:string[];theme:'system'|'light'|'dark';ui_language:string;excluded_apps:string[]};
export type Snapshot = {preferences:Preferences;paused:boolean;warning:string|null};
export const defaults:Preferences = {schema_version:1,delay_ms:300,languages:['it'],theme:'dark',ui_language:'it',excluded_apps:[]};
export const desktop = isTauri();
// Browser preview is explicitly opt-in, transient, and never touches real user preferences.
export const preview = !desktop && new URLSearchParams(location.search).get('preview') === '1';
let sample:Snapshot = {preferences:structuredClone(defaults),paused:false,warning:null};
export async function read():Promise<Snapshot> {
  if(desktop) return invoke('get_settings');
  if(preview) return structuredClone(sample);
  throw new Error('Apri Accento dall’app desktop. Anteprima: aggiungi ?preview=1 all’indirizzo.');
}
export async function save(preferences:Preferences):Promise<Snapshot> {
  if(desktop) return invoke('save_settings',{preferences});
  if(!preview) throw new Error('Desktop connection required');
  sample.preferences = structuredClone(preferences);return structuredClone(sample);
}
export async function pause(paused:boolean):Promise<Snapshot> {
  if(desktop) return invoke('set_paused',{paused});
  if(!preview) throw new Error('Desktop connection required');
  sample.paused = paused;return structuredClone(sample);
}
export async function subscribe(callback:(state:Snapshot)=>void) {
  return desktop ? listen<Snapshot>('accento-state',event=>callback(event.payload)) : ()=>{};
}
