import {test} from 'node:test';
import assert from 'node:assert/strict';
import {fixture} from '../src/fixtures.ts';
import {widgetPalettes,pressure,widgetSummary,mcpConfiguration} from '../src/widget-model.ts';
function luminance(hex:string){const c=hex.slice(1).match(/../g)!.map(v=>parseInt(v,16)/255).map(v=>v<=0.04045?v/12.92:((v+0.055)/1.055)**2.4);return .2126*c[0]!+.7152*c[1]!+.0722*c[2]!;}
function contrast(a:string,b:string){const values=[luminance(a),luminance(b)].sort((x,y)=>y-x);return (values[0]!+.05)/(values[1]!+.05);}
test('every widget foreground passes text contrast before palette acceptance',()=>{
  for(const [theme,p] of Object.entries(widgetPalettes))for(const bg of [p.surface,p.card]){
    for(const key of ['text','muted','accent','gpu','warning','critical'] as const){const value=contrast(p[key],bg);assert.ok(value>=4.5,`${theme} ${key} on ${bg}: ${value}`);console.log(`contrast ${theme} ${key} ${bg}: ${value.toFixed(2)}:1`);}
    assert.ok(contrast(p.border,bg)>=3,`${theme} control border`);
  }
});
test('resource pressure is bounded and not a health score or thermal threshold',()=>{
  for(const value of [null,NaN,Infinity,-1,101])assert.equal(pressure(value),'unknown');
  assert.equal(pressure(84.9),'measured');assert.equal(pressure(85),'elevated');assert.equal(pressure(95),'high');
  const s=fixture(1);s.cpu.temperature_c.value=110;s.cpu.temperature_c.availability='available';
  assert.equal(widgetSummary(s,null).state,'unknown');
});
test('widget never replaces missing or stale sensors with zero',()=>{
  const s=fixture(1);s.cpu.usage.age_ms=16000;
  const m=widgetSummary(s,null);assert.equal(m.cpu,null);assert.equal(m.temperature,null);assert.equal(m.gpu,null);
  assert.equal(widgetSummary(null,null).ram,null);
});
test('process observations are bounded, memory-ranked and treated as text',()=>{
  const s=fixture(1);s.processes=[{name:'<script>\u0000',memory_bytes:1,cpu_availability:'stale'}, {name:'model',memory_bytes:9,cpu_percent:5,cpu_availability:'available'}];
  const p=widgetSummary(s,null).processes;assert.equal(p[0]?.name,'model');assert.equal(p[1]?.name,'<script>');assert.equal(p[1]?.cpu,null);
  assert.equal((s.processes[0] as any).name,'<script>\u0000');
});
test('MCP setup is config data, excludes process opt-in and rejects injected paths',()=>{
  const path='/opt/System Pulse/systempulse-mcp';
  const c=JSON.parse(mcpConfiguration('claude',path));assert.deepEqual(c.mcpServers.systempulse,{command:path,args:[]});
  assert.match(mcpConfiguration('codex',path),/^\[mcp_servers.systempulse\]/);
  assert.ok(!mcpConfiguration('codex',path).includes('share-processes'));
  for(const value of ['relative','/tmp/a\ncommand="bad"','/tmp/\u0000'])assert.throws(()=>mcpConfiguration('codex',value));
});
