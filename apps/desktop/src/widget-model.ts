import {available,ratio,safeText,isMetric,type Snapshot,type HealthReport} from './model.ts';

// Opaque surfaces: measured contrast must not depend on the desktop wallpaper.
export const widgetPalettes = {
  dark: {surface:'#111c2b',card:'#19283a',text:'#edf4ff',muted:'#b1c0d2',accent:'#73d9ca',gpu:'#a9c7ff',warning:'#ffcf80',critical:'#ff9cab',border:'#71849b'},
  light:{surface:'#f5f8fc',card:'#e8eff6',text:'#14283d',muted:'#475d73',accent:'#0b655f',gpu:'#335fa8',warning:'#865000',critical:'#a7193c',border:'#718198'},
} as const;
export type Pressure = 'unknown'|'measured'|'elevated'|'high';
export function pressure(value:number|null):Pressure {
  if(value===null||!Number.isFinite(value)||value<0||value>100)return 'unknown';
  return value>=95?'high':value>=85?'elevated':'measured';
}
export function widgetSummary(snapshot:Snapshot|null,report:HealthReport|null){
  const gpuValues=(snapshot?.gpu??[]).flatMap(g=>{
    const usage=(g as Record<string,unknown>).usage;
    const v=isMetric(usage)?available(usage):null;
    return v===null||v<0||v>100?[]:[v];
  });
  const ram=snapshot?ratio(snapshot.memory.used_bytes,snapshot.memory.total_bytes):null;
  return {ram,ramPressure:pressure(ram),cpu:snapshot?available(snapshot.cpu.usage):null,
    temperature:snapshot?available(snapshot.cpu.temperature_c):null,
    gpu:gpuValues.length?Math.max(...gpuValues):null,
    state:report?.state??'unknown',
    processes:(snapshot?.processes??[]).map(p=>p as Record<string,unknown>)
      .sort((a,b)=>Number(b.memory_bytes)-Number(a.memory_bytes)).slice(0,3)
      .map(p=>({name:safeText(p.name,64),memory:Number(p.memory_bytes),cpu:p.cpu_availability==='available'?Number(p.cpu_percent):null})),
  };
}
export function mcpConfiguration(client:'codex'|'claude',command:string):string {
  if(command.length>4096||/[\u0000-\u001f\u007f]/.test(command)||!(/^(\/|[A-Za-z]:[\\/])/.test(command)))throw new Error('Invalid MCP executable path');
  return client==='codex'
    ? `[mcp_servers.systempulse]\ncommand = ${JSON.stringify(command)}\nargs = []\n`
    : JSON.stringify({mcpServers:{systempulse:{command,args:[]}}},null,2);
}
