import { available, filterProcesses, ratio, safeText, type Metric, type Snapshot, type HealthReport } from './model';
import { messages, type Key, type Language } from './i18n';
export function el<K extends keyof HTMLElementTagNameMap>(tag: K, className = '', text?: string): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag); node.className = className; if (text !== undefined) node.textContent = text; return node;
}
export function format(value: number | null, unit = '%'): string {
  if (value === null || !Number.isFinite(value)) return '—';
  if (unit === 'bytes') return `${(value / 1024 ** 3).toFixed(1)} GiB`;
  return `${value.toFixed(1)}${unit}`;
}
export function panel(title: string, hint?: string): HTMLElement {
  const p = el('section','panel'); p.append(el('h2','',title)); if (hint) p.append(el('p','muted',hint)); return p;
}
export function metricCard(title: string, metric: Metric | undefined, language: Language, unit = '%'): HTMLElement {
  const t = messages[language], card = el('section','metric');
  const keys:Record<string,string>={'cpu.usage':'cpu','memory.used':'ram','cpu.temperature':'temperature','swap.used':'swap'};
  if(metric&&keys[metric.id])card.dataset.measureKey=keys[metric.id];
  card.append(el('h2','',title));
  card.append(el('p','metric-value',format(metric ? available(metric) : null,unit)));
  card.append(el('span','muted',metric?.availability === 'available' ? safeText(metric.source) : t.unavailable)); return card;
}
export function renderOverview(snapshot: Snapshot | null, language: Language): HTMLElement {
  const t=messages[language], body=el('div','dashboard'), cards=el('div','metrics');
  cards.append(metricCard(t.cpu,snapshot?.cpu.usage,language),metricCard(t.memory,snapshot?.memory.used_bytes,language,'bytes'),metricCard(t.temperature,snapshot?.cpu.temperature_c,language,' °C'),metricCard(t.swap,snapshot?.memory.swap_used_bytes,language,'bytes')); body.append(cards);
  const chart=panel(t.activity,t.activityHint); chart.append(el('div','chart-host')); body.append(chart);
  const details=el('div','detail-grid');
  const system=panel(t.observations); system.append(el('p','',`${snapshot?.cpu.logical_cores ?? '—'} ${t.cores}`)); system.append(el('p','muted',`${t.frequency}: ${format(snapshot ? available(snapshot.cpu.frequency_mhz) : null,' MHz')}`));
  system.append(el('p','muted',`${t.memory}: ${snapshot ? format(ratio(snapshot.memory.used_bytes,snapshot.memory.total_bytes)) : '—'}`)); details.append(system);
  const storage=panel(t.storage);
  if (!snapshot?.storage.length) storage.append(el('p','muted',t.none));
  for (const item of snapshot?.storage.slice(0,8) ?? []) {
    const disk=item as Record<string,unknown>, row=el('div','storage-row'); row.append(el('strong','',safeText(disk.name ?? disk.id)));
    const total=disk.total_bytes, free=disk.available_bytes;
    row.append(el('span','muted',`${total && typeof total==='object' ? format(available(total as Metric),'bytes') : '—'} · ${free && typeof free==='object' ? format(available(free as Metric),'bytes') : '—'} ${t.free}`)); storage.append(row);
  }
  details.append(storage);
  for (const [title,items] of [[t.gpu,snapshot?.gpu],[t.battery,snapshot?.batteries]] as const) { const p=panel(title); p.append(el('p','muted',items?.length ? `${items.length} · ${t.notMeasured}` : t.none)); details.append(p); }
  body.append(details); return body;
}
export function renderProcesses(snapshot: Snapshot | null, language: Language,liveSnapshot:()=>Snapshot|null=()=>snapshot,afterDraw?:(root:HTMLElement)=>void): HTMLElement {
  const t=messages[language], body=panel(t.processes,t.privacy), input=el('input'); input.type='search';input.maxLength=120;input.placeholder=t.search;input.setAttribute('aria-label',t.search);body.append(input);
  const wrap=el('div','table-wrap'), table=el('table'), head=el('thead'), header=el('tr');
  for (const title of [t.name,t.pid,'CPU',t.ram]) header.append(el('th','',title)); head.append(header); const tbody=el('tbody'); table.append(head,tbody);wrap.append(table);body.append(wrap);
  const draw=()=>{ tbody.replaceChildren();for (const process of filterProcesses(liveSnapshot()?.processes ?? [],input.value)) { const row=el('tr');for (const [i,value] of [safeText(process.name),String(process.pid ?? '—'),format(process.cpu_availability==='available'&&typeof process.cpu_percent==='number'?process.cpu_percent:null),format(typeof process.memory_bytes==='number'?process.memory_bytes:null,'bytes')].entries()) {const cell=el('td','',value);if(i>=2)cell.dataset.measureKey=`process:${process.pid}:${i===2?'cpu':'memory'}`;row.append(cell);}tbody.append(row); }afterDraw?.(body); };
  body.addEventListener('systempulse:refresh',draw);input.addEventListener('input',draw);draw();return body;
}
export function renderStatic(page: Key, language: Language): HTMLElement {
  const t=messages[language];
  if (page==='diagnostics') { const p=panel(t.observations,t.diagnosticHint);p.append(el('p','notice',t.unknown),el('p','muted',t.evidence),el('p','',t.noDiagnostics));return p; }
  if (page==='ai') { const p=panel(t.mcp,t.mcpHint);p.append(el('p','notice',t.mcpUnknown),el('h3','',t.tools),el('p','',t.sharing));p.append(el('code','tool-list','get_system_health · get_cpu_telemetry · get_memory_status · get_gpu_status · get_storage_status · get_top_processes · get_active_alerts · get_health_history · get_optimization_advice · get_collector_status'));return p; }
  return panel(t.history,t.historyHint);
}

export function renderDiagnostics(report: HealthReport | null, language: Language): HTMLElement {
  const t=messages[language], p=panel(t.diagnostics,t.diagnosticHint);
  p.append(el('p','notice',report ? safeText(report.state) : t.unknown));
  for (const alert of report?.alerts ?? []) {
    const section=el('section','diagnostic');section.append(el('h3','',safeText(alert.title)),el('p','',safeText(alert.evidence,500)),el('p','muted',safeText(alert.consequence,500)),el('p','',safeText(alert.recommendation,500)),el('p','muted',safeText(alert.action_risk,500)));p.append(section);
  }
  if(report?.alerts.length===0)p.append(el('p','muted',language==='fr'?'Aucune alerte active sur les règles évaluées.':'No active alerts for the evaluated rules.'));
  for(const reason of report?.missing_evidence ?? [])p.append(el('p','muted',safeText(reason,500)));
  return p;
}
