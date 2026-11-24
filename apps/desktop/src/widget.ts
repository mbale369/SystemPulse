import {invoke} from '@tauri-apps/api/core';
import {demoMode,snapshot,health} from './backend';
import {el} from './view';
import {widgetSummary,widgetPalettes} from './widget-model';
import {loadPreferences,resolveLanguage,resolveTheme} from './preferences';
import {renderAgentSetup,type AgentPreparation} from './agent-setup';
import type {Snapshot,HealthReport} from './model';
import './widget.css';
import {MeasurementChanges,decorateChanges,changeText,DragGesture,type ChangeFrame} from './measurement-changes';
import {SubtleAudio,soundEnabled,saveSound} from './subtle-audio';
import './changes.css';

const words={
  fr:{expand:'Afficher les détails',collapse:'Réduire les détails',move:'Déplacer la capsule',heat:'Température CPU',unknown:'Non mesuré',measured:'Mesures disponibles',elevated:'RAM élevée',high:'RAM très élevée',partial:'Couverture partielle',offline:'Collecteur indisponible',dashboard:'Tableau de bord',hide:'Masquer la capsule',quit:'Quitter SystemPulse',observed:'Consommateurs observés · RAM',processHint:'Noms locaux uniquement. GPU par processus non mesuré. Classement parmi les processus observés.',thermalHint:'Pas de seuil thermique matériel qualifié : cette valeur seule ne confirme pas une surchauffe.',gpuHint:'Charge GPU maximale parmi les capteurs disponibles.',live:'Observations locales · cadence 2–5 s',demo:'DÉMONSTRATION · données fictives',error:'Action indisponible sur ce bureau',agents:'Connexion IA'},
  en:{expand:'Show details',collapse:'Collapse details',move:'Move capsule',heat:'CPU temperature',unknown:'Not measured',measured:'Measurements available',elevated:'High RAM use',high:'Very high RAM use',partial:'Partial coverage',offline:'Collector unavailable',dashboard:'Dashboard',hide:'Hide capsule',quit:'Quit SystemPulse',observed:'Observed consumers · RAM',processHint:'Local names only. Per-process GPU not measured. Ranked among observed processes.',thermalHint:'No qualified hardware thermal threshold: this reading alone does not confirm overheating.',gpuHint:'Maximum GPU load among available sensors.',live:'Local observations · 2–5 s cadence',demo:'DEMONSTRATION · simulated data',error:'Action unavailable on this desktop',agents:'AI connection'},
};
export function mountWidget():void {
  document.body.classList.add('widget-mode');
  const app=document.querySelector<HTMLElement>('#app');if(!app)throw new Error('Missing root');
  document.querySelector('.skip')?.remove();
  const media=window.matchMedia('(prefers-color-scheme: dark)');
  const preferences=()=>{try{return loadPreferences(localStorage);}catch{return loadPreferences();}};
  let prefs=preferences(),language=resolveLanguage(prefs.language,navigator.languages),theme=resolveTheme(prefs.theme,media.matches);
  let expanded=false,pending=false,stopped=false,failures=0,timer:ReturnType<typeof setTimeout>|undefined;
  let data:Snapshot|null=null,report:HealthReport|null=null;
  const changes=new MeasurementChanges(),audio=new SubtleAudio(),gesture=new DragGesture();
  let frame:ChangeFrame={trends:new Map(),events:[],audible:false},sounds=true;
  try{sounds=soundEnabled(localStorage);}catch{}
  const agent:AgentPreparation={enabled:false,client:'codex'};
  const root=el('main','pulse-widget'),panel=el('section','widget-panel'),capsule=el('div','pulse-capsule');
  const toggle=el('button','capsule-toggle');toggle.type='button';
  toggle.setAttribute('aria-controls','widget-details');toggle.setAttribute('aria-expanded','false');panel.id='widget-details';panel.hidden=true;
  const ring=el('span','ram-ring'),ram=el('strong','ram-value','—'),summary=el('span','capsule-summary');
  const svg=document.createElementNS('http://www.w3.org/2000/svg','svg');svg.setAttribute('viewBox','0 0 64 64');svg.setAttribute('aria-hidden','true');
  const track=document.createElementNS(svg.namespaceURI,'circle'),arc=document.createElementNS(svg.namespaceURI,'circle');
  for(const circle of [track,arc]){circle.setAttribute('cx','32');circle.setAttribute('cy','32');circle.setAttribute('r','27');circle.setAttribute('fill','none');circle.setAttribute('stroke-width','4');}
  track.setAttribute('class','ring-track');arc.setAttribute('class','ring-progress');arc.setAttribute('pathLength','100');arc.setAttribute('stroke-linecap','round');svg.append(track,arc);
  const title=el('span','capsule-title','RAM'),state=el('span','capsule-state'),temp=el('span','capsule-temp');ring.append(svg,ram);summary.append(title,state);toggle.append(ring,summary,temp);
  capsule.append(toggle);root.append(panel,capsule);app.replaceChildren(root);
  const panelHeading=el('h1','','SystemPulse'),panelMeta=el('p','widget-muted'),metrics=el('div','widget-metrics'),thermal=el('p','widget-thermal');
  const cpu=el('strong'),gpu=el('strong'),ramDetail=el('strong');
  for(const [name,value,kind] of [['RAM',ramDetail,'ram'],['CPU',cpu,'cpu'],['GPU',gpu,'gpu']] as const){const card=el('section',`widget-metric ${kind}`);card.dataset.measureKey=kind;card.append(el('h2','',name),value);metrics.append(card);}
  thermal.dataset.measureKey='temperature';
  const healthEvidence=el('p','widget-health-evidence'),heatHint=el('p','widget-muted'),gpuHint=el('p','widget-muted'),processTitle=el('h2',''),processList=el('ul','widget-processes'),processHint=el('p','widget-muted');
  const agentHost=el('div'),buttons=el('div','widget-actions'),status=el('p','widget-action-status');status.setAttribute('role','status');
  const dashboard=el('button'),hide=el('button'),quit=el('button');
  for(const button of [dashboard,hide,quit]){button.type='button';buttons.append(button);}
  const changeFeed=el('aside','change-feed');changeFeed.setAttribute('role','status');
  const mute=el('button'),audioStatus=el('p','widget-muted'),moveHint=el('p','widget-muted');mute.type='button';
  mute.addEventListener('click',()=>{sounds=!sounds;try{saveSound(sounds,localStorage);}catch{}if(!sounds)audio.mute();else void audio.unlock().then(updateAudio);updateAudio();});
  function updateAudio(){mute.textContent=language==='fr'?(sounds?'Couper le son':'Activer le son'):(sounds?'Mute sound':'Enable sound');mute.setAttribute('aria-pressed',String(!sounds));audioStatus.textContent=!sounds?(language==='fr'?'Son coupé.':'Sound muted.'):audio.isReady()?(language==='fr'?'Son discret prêt · au maximum une fois toutes les 30 s.':'Subtle sound ready · at most once every 30 s.'):(language==='fr'?'Premier clic requis pour l’audio ; peut être bloqué par le bureau.':'Audio needs a first click; desktop policy may block it.');}
  panel.append(panelHeading,panelMeta,changeFeed,healthEvidence,metrics,thermal,heatHint,gpuHint,processTitle,processList,processHint,agentHost,mute,audioStatus,moveHint,buttons,status);
  const action=async(name:string)=>{try{if(!demoMode)await invoke('widget_action',{action:name});else if(name==='dashboard')location.href='/';else if(name==='hide')root.hidden=true;else status.textContent=language==='fr'?'Fermez cet onglet de démonstration.':'Close this demo tab.';}catch{status.textContent=words[language].error;}};
  dashboard.addEventListener('click',()=>{void action('dashboard');});hide.addEventListener('click',()=>{void action('hide');});quit.addEventListener('click',()=>{void action('quit');});
  const changeExpanded=async(next:boolean)=>{
    if(pending||next===expanded)return;pending=true;toggle.disabled=true;
    try{if(!demoMode)await invoke('widget_expand',{expanded:next});expanded=next;panel.hidden=!expanded;root.classList.toggle('expanded',expanded);toggle.setAttribute('aria-expanded',String(expanded));toggle.setAttribute('aria-label',expanded?words[language].collapse:words[language].expand);}
    catch{status.textContent=words[language].error;}finally{pending=false;toggle.disabled=false;if(!expanded)toggle.focus();}
  };
  toggle.addEventListener('click',event=>{if(gesture.allowClick(event.detail)){if(sounds)void audio.unlock().then(updateAudio);void changeExpanded(!expanded);}else{event.preventDefault();event.stopPropagation();}});
  capsule.addEventListener('pointerdown',event=>{if(event.button!==0||!event.isPrimary)return;gesture.start(event.pointerId,event.clientX,event.clientY);toggle.setPointerCapture(event.pointerId);if(sounds)void audio.unlock().then(updateAudio);});
  capsule.addEventListener('pointermove',event=>{if((event.buttons&1)&&gesture.move(event.pointerId,event.clientX,event.clientY)){event.preventDefault();if(!demoMode)void invoke('widget_drag').catch(()=>{status.textContent=words[language].error;});}});
  const finishDrag=(event:PointerEvent)=>{gesture.finish(event.pointerId);if(toggle.hasPointerCapture(event.pointerId))toggle.releasePointerCapture(event.pointerId);};
  capsule.addEventListener('pointerup',finishDrag);capsule.addEventListener('pointercancel',finishDrag);
  const keyboard=(event:KeyboardEvent)=>{if(event.key==='Escape'&&expanded){event.preventDefault();void changeExpanded(false);}};
  window.addEventListener('keydown',keyboard);
  function labels(){
    const t=words[language];document.documentElement.lang=language;document.documentElement.dataset.theme=theme;document.documentElement.style.colorScheme=theme;
    for(const [key,value]of Object.entries(widgetPalettes[theme]))root.style.setProperty(`--pulse-${key}`,value);
    toggle.setAttribute('aria-label',expanded?t.collapse:t.expand);toggle.title=language==='fr'?'Clic : détails · glisser : déplacer':'Click: details · drag: move';
    panelMeta.textContent=demoMode?t.demo:t.live;heatHint.textContent=t.thermalHint;gpuHint.textContent=t.gpuHint;
    processTitle.textContent=t.observed;processHint.textContent=t.processHint;
    dashboard.textContent=t.dashboard;hide.textContent=t.hide;quit.textContent=t.quit;
    agentHost.replaceChildren(renderAgentSetup(language,agent));
    moveHint.textContent=language==='fr'?'Glissez sur la capsule pour la déplacer ; cliquez pour les détails. Au clavier : Entrée et Échap.':'Drag anywhere on the capsule to move; click for details. Keyboard: Enter and Escape.';updateAudio();
  }
  function paint(){
    const t=words[language],s=widgetSummary(data,report),percent=(v:number|null)=>v===null?'—':`${Math.round(v)}%`;
    ram.textContent=percent(s.ram);ramDetail.textContent=percent(s.ram);cpu.textContent=percent(s.cpu);gpu.textContent=percent(s.gpu);
    const temperature=s.temperature===null?'— °C':`${s.temperature.toFixed(0)} °C`;
    temp.textContent=temperature;temp.title=t.heat;thermal.textContent=`${t.heat} · ${s.temperature===null?t.unknown:temperature}`;
    const evaluatedAlert=s.state==='critical'||s.state==='warning';
    const alertLabel=language==='fr'?(s.state==='critical'?'Alerte critique évaluée':'Alerte évaluée'):(s.state==='critical'?'Evaluated critical alert':'Evaluated alert');
    const stateText=data===null?t.offline:evaluatedAlert?alertLabel:s.ramPressure==='high'?t.high:s.ramPressure==='elevated'?t.elevated:s.temperature===null||s.gpu===null||s.state==='unknown'?t.partial:t.measured;
    state.textContent=stateText;
    healthEvidence.hidden=!evaluatedAlert;healthEvidence.textContent=evaluatedAlert?`${alertLabel} · ${report?.alerts.slice(0,2).map(a=>a.title).join(' · ')??''}`:'';
    root.dataset.health=s.state;
    root.dataset.pressure=data===null?'unknown':s.ramPressure;
    arc.setAttribute('stroke-dasharray',`${s.ram??0} 100`);arc.setAttribute('opacity',s.ram===null||s.ram===0?'0':'1');
    title.textContent=s.ram===null?'RAM · —':'RAM';
    processList.replaceChildren();
    for(const p of s.processes){const row=el('li');row.append(el('span','',p.name),el('strong','',`${(p.memory/1024**3).toFixed(1)} GiB`),el('span','widget-muted',`CPU ${percent(p.cpu)}`));processList.append(row);}
    if(!s.processes.length)processList.append(el('li','widget-muted',t.unknown));
    decorateChanges(panel,frame,language);const message=changeText(frame,language);if(changeFeed.textContent!==message)changeFeed.textContent=message;
    capsule.dataset.notice=String(frame.events.length>0);
    const ramTrend=frame.trends.get('ram');title.textContent=`RAM${ramTrend?.direction==='up'?' ↑':ramTrend?.direction==='down'?' ↓':''}`;
  }
  const preferencesChanged=()=>{
    try{sounds=soundEnabled(localStorage);}catch{}if(!sounds)audio.mute();updateAudio();
    prefs=preferences();const nextLanguage=resolveLanguage(prefs.language,navigator.languages),nextTheme=resolveTheme(prefs.theme,media.matches);
    if(nextLanguage!==language||nextTheme!==theme){language=nextLanguage;theme=nextTheme;labels();paint();}
  };
  media.addEventListener('change',preferencesChanged);window.addEventListener('languagechange',preferencesChanged);window.addEventListener('storage',preferencesChanged);
  async function poll(){
    if(stopped)return;
    try{const next=await snapshot(),diagnostics=await health();if(stopped)return;data=next;report=diagnostics;frame=changes.sample(next,diagnostics,performance.now());if(frame.audible)audio.play(sounds,performance.now());failures=0;}
    catch{if(stopped)return;data=null;report=null;frame=changes.sample(null,null,performance.now());failures=Math.min(4,failures+1);}
    paint();if(!stopped)timer=setTimeout(()=>{void poll();},Math.min(30000,3000*2**failures));
  }
  window.addEventListener('pagehide',()=>{stopped=true;clearTimeout(timer);audio.close();media.removeEventListener('change',preferencesChanged);window.removeEventListener('storage',preferencesChanged);window.removeEventListener('languagechange',preferencesChanged);window.removeEventListener('keydown',keyboard);},{once:true});
  labels();paint();void poll();
}
