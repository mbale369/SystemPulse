import uPlot from 'uplot';
import 'uplot/dist/uPlot.min.css';
import './style.css';
import { configure, demoMode, snapshot, health } from './backend';
import { HISTORY_LIMIT, pushHistory, type Point, type Snapshot, type HealthReport } from './model';
import { messages, navigation, preferenceMessages, type Key } from './i18n';
import { loadPreferences, savePreferences, resolveLanguage, resolveTheme } from './preferences';
import { el, panel, renderOverview, renderProcesses, renderStatic, renderDiagnostics } from './view';
import {mountWidget} from './widget';
import {renderAgentSetup,type AgentPreparation} from './agent-setup';
import {invoke} from '@tauri-apps/api/core';
import {MeasurementChanges,decorateChanges,changeText,type ChangeFrame} from './measurement-changes';
import {soundEnabled,saveSound,SubtleAudio} from './subtle-audio';
import './changes.css';
if(new URLSearchParams(location.search).get('widget')==='1'){
  mountWidget();
}else{
const app=document.querySelector<HTMLDivElement>('#app');
if (!app) throw new Error('Missing application root');
let preferences=loadPreferences();
try { preferences=loadPreferences(localStorage); } catch { /* Storage may be unavailable. */ }
const systemAppearance=window.matchMedia('(prefers-color-scheme: dark)');
const systemLanguages=()=>navigator.languages.length ? navigator.languages : [navigator.language];
let language=resolveLanguage(preferences.language,systemLanguages()), theme=resolveTheme(preferences.theme,systemAppearance.matches), profile='balanced';
let page: Key='overview', data: Snapshot|null=null, error=false, busy=true, stopped=false, timer:ReturnType<typeof setTimeout>|undefined, failures=0;
const history:Point[]=[];
const changes=new MeasurementChanges();let changeFrame:ChangeFrame={trends:new Map(),events:[],audible:false};
let sounds=true;try{sounds=soundEnabled(localStorage);}catch{/* default */}
const soundPreview=new SubtleAudio(),changeFeed=el('aside','change-feed');changeFeed.setAttribute('role','status');
const agentPreparation:AgentPreparation={enabled:false,client:'codex'};
let report:HealthReport|null=null;
let chart:uPlot|null=null, observer:ResizeObserver|null=null;
const nav=el('nav'),content=el('main'),status=el('span','collector-state');content.id='content';content.tabIndex=-1;
function t(key:Key):string{return messages[language][key];}
function drawShell():void {
  document.documentElement.lang=language;document.documentElement.dataset.theme=theme;document.documentElement.style.colorScheme=theme;app!.replaceChildren();
  const sidebar=el('aside','sidebar'),brand=el('div','brand');brand.append(el('span','brand-mark','S'),el('strong','','SystemPulse'));sidebar.append(brand);nav.replaceChildren();nav.setAttribute('aria-label',language==='fr'?'Navigation principale':'Main navigation');
  for(const key of navigation){const button=el('button','nav-item',t(key));button.type='button';button.dataset.page=key;button.setAttribute('aria-current',page===key?'page':'false');button.addEventListener('click',()=>{page=key;drawShell();const active=nav.querySelector<HTMLButtonElement>(`[data-page="${key}"]`);active?.focus();});nav.append(button);}
  sidebar.append(nav,el('p','local-note',t('local')));
  const workspace=el('div','workspace'),header=el('header','topbar'),widgetButton=el('button','button',language==='fr'?'Capsule de surveillance':'Monitor capsule');widgetButton.type='button';widgetButton.addEventListener('click',()=>{if(demoMode){location.href='/?widget=1';}else void invoke('show_widget').catch(()=>{error=true;render();});});header.append(el('span','breadcrumb',`SystemPulse / ${t(page)}`),widgetButton,status);workspace.append(header,changeFeed,content);app!.append(sidebar,workspace);render();
}
function drawChart():void {
  const host=content.querySelector<HTMLElement>('.chart-host');if(!host)return;
  if(history.length<2){host.append(el('p','muted',t('emptyHistory')));return;}
  const bright=theme==='light',axis=bright?'#5c6674':'#a2adba';
  chart=new uPlot({width:Math.max(220,host.clientWidth),height:230,scales:{x:{time:true},y:{range:[0,100]}},legend:{show:true},cursor:{show:true},series:[{}, {label:t('cpu'),stroke:'#37cbb2',width:2}, {label:t('memory'),stroke:bright?'#336caf':'#8ab4e8',width:2}],axes:[{stroke:axis,grid:{show:false}},{stroke:axis,grid:{stroke:bright?'#e5e8ed':'#29313d'},values:(_u,ticks)=>ticks.map(v=>`${v}%`)}]},[history.map(p=>p.timestamp/1000),history.map(p=>p.cpu),history.map(p=>p.memory)],host);
  observer=new ResizeObserver(()=>{if(chart)chart.setSize({width:Math.max(220,host.clientWidth),height:230});});observer.observe(host);
}
function render():void {
  chart?.destroy();chart=null;observer?.disconnect();observer=null;content.replaceChildren();
  status.textContent=error?t('error'):busy?t('connecting'):t('live');status.dataset.state=error?'error':'active';
  const intro=el('div','intro');intro.append(el('p','eyebrow',t(page)),el('h1','',page==='overview'?t('title'):t(page)),el('p','muted',t('subtitle')));
  content.append(intro);if(demoMode)content.append(el('p','demo-banner',t('demo')));
  if(error){const errorBox=el('p','notice',t('error'));errorBox.setAttribute('role','status');content.append(errorBox);}
  if(page==='overview')content.append(renderOverview(data,language));
  else if(page==='processes')content.append(renderProcesses(data,language,()=>data,root=>decorateChanges(root,changeFrame,language)));
  else if(page==='settings')drawSettings();
  else if(page==='diagnostics')content.append(renderDiagnostics(report,language));
  else if(page==='ai')content.append(renderAgentSetup(language,agentPreparation));
  else {content.append(renderStatic(page,language));if(page==='history'){content.append(el('div','chart-host'));drawExport();}}
  if(page==='overview'||page==='history')drawChart();
  decorateChanges(content,changeFrame,language);const message=changeText(changeFrame,language);if(changeFeed.textContent!==message)changeFeed.textContent=message;
}
function drawSettings():void {
  const p=panel(t('preferences')),form=el('div','settings-grid');
  const select=(key:Key,options:[string,string][],value:string,change:(value:string)=>void)=>{const label=el('label','setting');label.append(el('span','',t(key)));const input=el('select');input.dataset.setting=key;input.setAttribute('aria-label',t(key));for(const [v,text]of options){const option=el('option','',text);option.value=v;input.append(option);}input.value=value;input.addEventListener('change',()=>change(input.value));label.append(input);form.append(label);};
  select('theme',[['system',preferenceMessages[language].system],['dark',t('dark')],['light',t('light')]],preferences.theme,value=>{preferences.theme=value==='dark'||value==='light'?value:'system';save();applyPreferences(true);});
  select('language',[['system',preferenceMessages[language].system],['fr','Français'],['en','English']],preferences.language,value=>{preferences.language=value==='en'||value==='fr'?value:'system';save();applyPreferences(true);});
  select('profile',[['eco',t('eco')],['balanced',t('balanced')],['detailed',t('detailed')]],profile,value=>{void configure(value).then(()=>{profile=value;}).catch(()=>{error=true;render();});});
  const soundLabel=el('label','sound-control'),soundSwitch=el('input');soundSwitch.type='checkbox';soundSwitch.checked=sounds;
  soundSwitch.dataset.soundControl='true';soundSwitch.addEventListener('change',()=>{sounds=soundSwitch.checked;try{saveSound(sounds,localStorage);}catch{}if(!sounds)soundPreview.mute();});
  soundLabel.append(soundSwitch,el('span','',language==='fr'?'Sons discrets des changements importants':'Subtle sounds for important changes'));
  const testSound=el('button','button',language==='fr'?'Tester le son':'Test sound');testSound.type='button';
  testSound.addEventListener('click',()=>{void soundPreview.unlock().then(ok=>{if(ok)soundPreview.play(true,performance.now(),true);else testSound.textContent=language==='fr'?'Audio indisponible':'Audio unavailable';});});
  p.append(soundLabel,testSound,el('p','muted',language==='fr'?'Un son court au maximum toutes les 30 s, émis par la capsule après un premier clic. Aucune sonnerie pour chaque fluctuation.':'At most one short sound every 30 s, emitted by the capsule after a first click. No sound for every fluctuation.'));
  p.append(form,el('p','muted',preferenceMessages[language].hint),el('p','muted',preferenceMessages[language].profileHint),el('p','muted',t('limited')));content.append(p);
}
function save():void{try{savePreferences(preferences,localStorage);}catch{/* Preferences remain in memory. */}}
function applyPreferences(force=false):void{
  const nextLanguage=resolveLanguage(preferences.language,systemLanguages()),nextTheme=resolveTheme(preferences.theme,systemAppearance.matches);
  if(!force&&nextLanguage===language&&nextTheme===theme)return;
  const focused=content.querySelector<HTMLSelectElement>('select:focus')?.dataset.setting;
  language=nextLanguage;theme=nextTheme;drawShell();
  if(focused)content.querySelector<HTMLSelectElement>(`select[data-setting="${focused}"]`)?.focus();
}
const onSystemPreferenceChange=()=>applyPreferences();
const onSoundStorage=()=>{try{sounds=soundEnabled(localStorage);}catch{}const input=content.querySelector<HTMLInputElement>('[data-sound-control]');if(input)input.checked=sounds;if(!sounds)soundPreview.mute();};
window.addEventListener('storage',onSoundStorage);
window.addEventListener('languagechange',onSystemPreferenceChange);
systemAppearance.addEventListener('change',onSystemPreferenceChange);
function drawExport():void {
  const p=panel(t('export'),t('exportHint')),download=el('button','button',t('download')),clear=el('button','button secondary',t('clear'));
  download.type='button';download.addEventListener('click',()=>{const url=URL.createObjectURL(new Blob([JSON.stringify({schema:1,source:demoMode?'demo':'local',points:history.slice(-HISTORY_LIMIT)},null,2)],{type:'application/json'}));const anchor=el('a');anchor.href=url;anchor.download='systempulse-session.json';anchor.click();setTimeout(()=>URL.revokeObjectURL(url),1000);});
  clear.type='button';clear.addEventListener('click',()=>{if(window.confirm(t('clearConfirm'))){history.length=0;render();}});p.append(download,clear);content.append(p);
}
async function poll():Promise<void>{
  if(stopped)return;
   try{const next=await snapshot();if(stopped)return;data=next;report=await health();changeFrame=changes.sample(next,report,performance.now());pushHistory(history,next);error=false;busy=false;failures=0;
    const message=changeText(changeFrame,language);if(changeFeed.textContent!==message)changeFeed.textContent=message;
    // Preserve keyboard focus and process search while the user edits.
    if(!content.contains(document.activeElement)||page!=='processes'&&page!=='settings'&&page!=='ai')render();else {status.textContent=t('live');if(page==='processes')content.querySelector('.panel')?.dispatchEvent(new Event('systempulse:refresh'));}
   }catch{if(stopped)return;error=true;data=null;report=null;changeFrame=changes.sample(null,null,performance.now());busy=false;failures=Math.min(failures+1,5);render();}
  const interval=profile==='eco'?5000:profile==='detailed'?2000:3000;
  if(!stopped)timer=setTimeout(()=>{void poll();},Math.min(30000,interval*2**failures));
}
window.addEventListener('pagehide',()=>{stopped=true;clearTimeout(timer);soundPreview.close();chart?.destroy();observer?.disconnect();window.removeEventListener('storage',onSoundStorage);window.removeEventListener('languagechange',onSystemPreferenceChange);systemAppearance.removeEventListener('change',onSystemPreferenceChange);},{once:true});
drawShell();void poll();
}
