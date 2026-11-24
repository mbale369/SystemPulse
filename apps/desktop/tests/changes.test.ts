import {test} from 'node:test';
import assert from 'node:assert/strict';
import {fixture} from '../src/fixtures.ts';
import {MeasurementChanges,DragGesture,trendText,changeText} from '../src/measurement-changes.ts';
import {SoundCooldown,soundEnabled,saveSound} from '../src/subtle-audio.ts';
function sample(sequence:number,cpu:number){const s=fixture(sequence);s.cpu.usage.value=cpu;return s;}
test('startup, cached sequences and a single spike never request sound',()=>{
  const c=new MeasurementChanges();assert.equal(c.sample(sample(1,10),null,0).audible,false);
  assert.equal(c.sample(sample(1,90),null,1000).audible,false);
  assert.equal(c.sample(sample(2,90),null,3000).audible,false);
  const result=c.sample(sample(3,10),null,6000);assert.equal(result.audible,false);assert.equal(result.events.length,0);
});
test('large persistent changes are confirmed and bounded by per-metric cooldown',()=>{
  const c=new MeasurementChanges();c.sample(sample(1,10),null,0);c.sample(sample(2,70),null,3000);
  const frame=c.sample(sample(3,70),null,6000);assert.equal(frame.audible,true);assert.equal(frame.events[0]?.key,'cpu');
  assert.match(changeText(frame,'fr'),/CPU en hausse/);
  c.sample(sample(4,10),null,9000);assert.equal(c.sample(sample(5,10),null,12000).audible,false);
  c.sample(sample(6,90),null,15000);assert.equal(c.sample(sample(7,90),null,18000).audible,false);
  assert.ok(frame.events.length<=4);
});
test('unavailable data and long gaps reset baselines without fake zero changes',()=>{
  const c=new MeasurementChanges();c.sample(sample(1,10),null,0);
  c.sample(null,null,3000);assert.equal(c.sample(sample(2,90),null,6000).audible,false);
  assert.equal(c.sample(sample(3,10),null,30000).audible,false);
  const stale=sample(4,90);stale.cpu.usage.age_ms=20000;
  assert.equal(c.sample(stale,null,33000).audible,false);
});
test('evaluated alerts notify only when new, not at startup or every refresh',()=>{
  const c=new MeasurementChanges(),report={state:'warning',alerts:[{id:'cpu_sustained'}],resolved:[],missing_evidence:[]} as any;
  assert.equal(c.sample(sample(1,10),report,0).audible,false);
  assert.equal(c.sample(sample(2,10),report,3000).audible,false);
  const newReport={...report,alerts:[...report.alerts,{id:'memory_pressure'}]};
  assert.equal(c.sample(sample(3,10),newReport,6000).audible,true);
  assert.equal(c.sample(sample(4,10),newReport,9000).audible,false);
});
test('trend text includes direction and units and never claims a hardware failure',()=>{
  assert.equal(trendText({delta:2,direction:'up',unit:'points'},'fr'),'↑ +2.0 pts');
  assert.equal(trendText({delta:-2,direction:'down',unit:'celsius'},'en'),'↓ −2.0 °C');
  assert.equal(trendText(undefined,'fr'),'');
});
test('full-surface drag differentiates click, threshold, pointer ids and keyboard',()=>{
  const g=new DragGesture();g.start(1,10,10);assert.equal(g.move(1,12,12),false);assert.equal(g.allowClick(1),true);
  assert.equal(g.move(2,40,40),false);assert.equal(g.move(1,30,10),true);assert.equal(g.move(1,40,10),false);
  g.finish(1);assert.equal(g.allowClick(1),false);assert.equal(g.allowClick(0),true);
  g.start(2,0,0);g.finish(2);assert.equal(g.allowClick(1),true);
});
test('sound cooldown and storage deny floods and tolerate unavailable storage',()=>{
  const gate=new SoundCooldown();assert.equal(gate.accept(0),true);assert.equal(gate.accept(1),false);assert.equal(gate.accept(29999),false);assert.equal(gate.accept(30000),true);
  assert.equal(gate.accept(NaN),false);assert.equal(gate.accept(-1),false);
  const values=new Map<string,string>(),storage={getItem:(k:string)=>values.get(k)??null,setItem:(k:string,v:string)=>{values.set(k,v);}};
  assert.equal(soundEnabled(storage),true);saveSound(false,storage);assert.equal(soundEnabled(storage),false);
  assert.doesNotThrow(()=>saveSound(true,{setItem:()=>{throw new Error('denied');}}));
});
