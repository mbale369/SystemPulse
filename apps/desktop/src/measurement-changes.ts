import {available,ratio,type Snapshot,type HealthReport} from './model.ts';
import {widgetSummary} from './widget-model.ts';
import type {Language} from './i18n.ts';
export interface Trend {delta:number;direction:'up'|'down'|'stable';unit:'points'|'celsius'|'bytes'}
export interface ChangeEvent {key:string;delta:number;kind:'change'|'alert';at:number}
export interface ChangeFrame {trends:Map<string,Trend>;events:ChangeEvent[];audible:boolean}
const thresholds:Record<string,number>={ram:8,cpu:20,gpu:20,temperature:5};
function values(s:Snapshot):Map<string,{value:number|null;unit:Trend['unit']}>{
  const summary=widgetSummary(s,null);
  const result=new Map<string,{value:number|null;unit:Trend['unit']}>([
    ['ram',{value:ratio(s.memory.used_bytes,s.memory.total_bytes),unit:'points'}],
    ['cpu',{value:available(s.cpu.usage),unit:'points'}],
    ['gpu',{value:summary.gpu,unit:'points'}],
    ['temperature',{value:available(s.cpu.temperature_c),unit:'celsius'}],
    ['swap',{value:available(s.memory.swap_used_bytes),unit:'bytes'}],
  ]);
  for(const raw of s.processes.slice(0,100)){
    const p=raw as Record<string,unknown>;
    result.set(`process:${p.pid}:cpu`,{value:p.cpu_availability==='available'?Number(p.cpu_percent):null,unit:'points'});
    result.set(`process:${p.pid}:memory`,{value:Number(p.memory_bytes),unit:'bytes'});
  }
  return result;
}
export class MeasurementChanges {
  private previous=new Map<string,{value:number|null;unit:Trend['unit']}>();
  private baseline=new Map<string,number>();
  private pending=new Map<string,number>();
  private cooldown=new Map<string,number>();
  private alerts=new Set<string>();
  private sequence:number|null=null;
  private at:number|null=null;
  private events:ChangeEvent[]=[];
  private trends=new Map<string,Trend>();
  sample(snapshot:Snapshot|null,report:HealthReport|null,now:number):ChangeFrame{
    const reset=this.at===null||now<this.at||now-this.at>15000;
    this.events=this.events.filter(e=>now>=e.at&&now-e.at<15000).slice(-4);
    if(!snapshot){this.sequence=null;this.at=null;this.previous.clear();this.pending.clear();this.baseline.clear();this.trends.clear();return {trends:this.trends,events:[],audible:false};}
    if(snapshot.sequence===this.sequence&&!reset)return {trends:this.trends,events:this.events,audible:false};
    const next=values(snapshot),newEvents:ChangeEvent[]=[];
    this.trends=new Map();
    const freshReset=reset||this.sequence===null||snapshot.sequence<this.sequence;
    if(freshReset){this.baseline.clear();this.pending.clear();this.events=[];}
    for(const [key,{value,unit}]of next){
      const previous=this.previous.get(key)?.value;
      if(value!==null&&Number.isFinite(value)&&previous!==null&&previous!==undefined&&!freshReset){
        const delta=value-previous,minimum=unit==='bytes'?16*1024**2:unit==='celsius'?.5:1;
        this.trends.set(key,{delta,direction:Math.abs(delta)<minimum?'stable':delta>0?'up':'down',unit});
      }
      if(!(key in thresholds))continue;
      if(value===null||!Number.isFinite(value)){this.baseline.delete(key);this.pending.delete(key);continue;}
      if(freshReset||!this.baseline.has(key)){this.baseline.set(key,value);continue;}
      const delta=value-this.baseline.get(key)!,direction=Math.sign(delta);
      if(Math.abs(delta)>=thresholds[key]!){
        if(this.pending.get(key)===direction){
          if(now-(this.cooldown.get(key)??-Infinity)>=60000){newEvents.push({key,delta,kind:'change',at:now});this.cooldown.set(key,now);}
          this.baseline.set(key,value);this.pending.delete(key);
        }else this.pending.set(key,direction);
      }else this.pending.delete(key);
    }
    const alertIds=new Set((report?.alerts??[]).map(a=>`${a.id}:${a.state}`));
    if(!freshReset&&[...alertIds].some(id=>!this.alerts.has(id))&&now-(this.cooldown.get('alert')??-Infinity)>=60000){
      newEvents.push({key:'alert',delta:0,kind:'alert',at:now});this.cooldown.set('alert',now);
    }
    this.alerts=alertIds;this.events=[...this.events,...newEvents].slice(-4);
    this.previous=next;this.sequence=snapshot.sequence;this.at=now;
    // Only increases or new evaluated alerts are audible; decreases remain visual.
    return {trends:this.trends,events:this.events,audible:newEvents.some(e=>e.kind==='alert'||e.delta>0)};
  }
}
export function trendText(trend:Trend|undefined,language:Language):string{
  if(!trend||trend.direction==='stable')return '';
  const delta=trend.unit==='bytes'?Math.abs(trend.delta)/1024**3:Math.abs(trend.delta);
  const unit=trend.unit==='bytes'?'GiB':trend.unit==='celsius'?'°C':language==='fr'?'pts':'pp';
  return `${trend.direction==='up'?'↑ +':'↓ −'}${delta.toFixed(1)} ${unit}`;
}
export function changeText(frame:ChangeFrame,language:Language):string{
  return frame.events.slice(-2).map(e=>e.kind==='alert'?(language==='fr'?'Nouvelle alerte évaluée':'New evaluated alert'):
    `${e.key==='temperature'?(language==='fr'?'Température CPU':'CPU temperature'):e.key.toUpperCase()} ${language==='fr'?(e.delta>0?'en hausse':'en baisse'):(e.delta>0?'increased':'decreased')} (${Math.abs(e.delta).toFixed(1)} ${e.key==='temperature'?'°C':language==='fr'?'points':'pp'})`).join(' · ');
}
export function decorateChanges(root:HTMLElement,frame:ChangeFrame,language:Language){
  for(const node of root.querySelectorAll<HTMLElement>('[data-measure-key]')){
    const trend=frame.trends.get(node.dataset.measureKey!);node.dataset.trend=trend?.direction??'stable';
    let indicator=node.querySelector<HTMLElement>('.measure-delta');
    if(!indicator){indicator=document.createElement('span');indicator.className='measure-delta';node.append(indicator);}
    indicator.textContent=trendText(trend,language);
  }
}
export class DragGesture {
  private origin:{id:number;x:number;y:number}|null=null;
  private dragged=false;
  start(id:number,x:number,y:number){this.origin={id,x,y};this.dragged=false;}
  move(id:number,x:number,y:number):boolean{
    if(!this.origin||this.origin.id!==id||this.dragged||Math.hypot(x-this.origin.x,y-this.origin.y)<6)return false;
    this.dragged=true;return true;
  }
  finish(id:number){if(this.origin?.id===id)this.origin=null;}
  allowClick(detail:number){return detail===0||!this.dragged;}
}
