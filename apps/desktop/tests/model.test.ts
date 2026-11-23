import { test } from 'node:test';
import assert from 'node:assert/strict';
import { fixture } from '../src/fixtures.ts';
import { validateHealthReport, isMetric } from '../src/model.ts';
import { available, ratio, pushHistory, safeText, validateSnapshot, filterProcesses, HISTORY_LIMIT, type Metric } from '../src/model.ts';
const metric=(value:number|null,availability='available'):Metric=>({id:'cpu',kind:'cpu',value,unit:'percent',timestamp_ms:1,source:'fake',availability:availability as Metric['availability'],quality:'valid',age_ms:0});
test('missing, stale and non-finite sensor values are never displayed as zero',()=>{assert.equal(available(metric(null)),null);assert.equal(available(metric(50,'stale')),null);assert.equal(available(metric(Infinity)),null);assert.equal(available(metric(0)),0);});
test('memory ratios reject invalid denominators and inconsistent readings',()=>{assert.equal(ratio(metric(4),metric(8)),50);assert.equal(ratio(metric(4),metric(0)),null);assert.equal(ratio(metric(9),metric(8)),null);});
test('history is bounded and duplicate samples are not appended',()=>{const history=[];for(let i=0;i<10000;i++){pushHistory(history,{timestamp_ms:i,sequence:i,cpu:{usage:metric(1)},memory:{used_bytes:metric(2),total_bytes:metric(4)}} as never);}assert.equal(history.length,HISTORY_LIMIT);assert.equal(history[0]?.timestamp,9400);pushHistory(history,{timestamp_ms:9999} as never);assert.equal(history.length,HISTORY_LIMIT);});
test('malformed backend snapshots fail closed',()=>{for(const value of [null,{},[],{timestamp_ms:1,cpu:{}}])assert.throws(()=>validateSnapshot(value));});
test('process output is bounded and control characters are removed',()=>{assert.equal(safeText('\u0000hello\nworld'),'helloworld');assert.equal(safeText('x'.repeat(500)).length,120);assert.equal(filterProcesses(Array.from({length:100},()=>({name:'Editor'})),'edit').length,50);assert.equal(filterProcesses([{name:'Browser'}],'Editor').length,0);});
test('complete fixtures pass but missing fields and null collection entries fail closed',()=>{
  assert.equal(validateSnapshot(fixture(1)).sequence,1);
  for(const mutate of [
    (s:any)=>{s.memory={};}, (s:any)=>{delete s.memory.used_bytes;},
    (s:any)=>{s.storage=[null];}, (s:any)=>{s.processes=[null];},
    (s:any)=>{s.gpu=[null];}, (s:any)=>{s.batteries=Array(33).fill({});},
    (s:any)=>{s.cpu.logical_cores=-1;}, (s:any)=>{s.sequence=-1;},
    (s:any)=>{s.processes[0].cpu_percent=101;}, (s:any)=>{s.processes[0].memory_bytes=-1;},
  ]) {const s=fixture(1);mutate(s);assert.throws(()=>validateSnapshot(s));}
});
test('metric ages and timestamps must be finite nonnegative safe integers',()=>{
  for(const value of [-1,NaN,Infinity,0.1,Number.MAX_SAFE_INTEGER+1]) {
    const m=metric(50);m.age_ms=value;assert.equal(isMetric(m),false);assert.equal(available(m),null);
    const s=fixture(1);s.timestamp_ms=value;assert.throws(()=>validateSnapshot(s));
  }
});
test('diagnostics reject malformed fields before rendering',()=>{
  const valid={state:'unknown',alerts:[],resolved:[],missing_evidence:[]};
  assert.deepEqual(validateHealthReport(valid),valid);
  for(const value of [null,{}, {...valid,missing_evidence:null}, {...valid,resolved:[{}]}, {...valid,alerts:[null]}, {...valid,alerts:[{}]}, {...valid,missing_evidence:['x'.repeat(4097)]}]) assert.throws(()=>validateHealthReport(value));
});
test('bounded deterministic adversarial corpus never bypasses required memory fields',()=>{
  for(let i=0;i<1000;i++) {const s=fixture(i);s.memory=([null,{},[],{used_bytes:metric(1)}][i%4]) as never;assert.throws(()=>validateSnapshot(s));}
});
