export type Availability = 'available' | 'unsupported' | 'permission_denied' | 'temporarily_unavailable' | 'stale' | 'sensor_error';
export interface Metric { id: string; kind: string; value: number | null; unit: string; timestamp_ms: number; source: string; availability: Availability; quality: string; age_ms: number }
export interface CpuTelemetry { usage: Metric; per_core: Metric[]; frequency_mhz: Metric; temperature_c: Metric; logical_cores: number }
export interface MemoryStatus { total_bytes: Metric; used_bytes: Metric; available_bytes: Metric; swap_total_bytes: Metric; swap_used_bytes: Metric }
export interface Snapshot { timestamp_ms: number; sequence: number; cpu: CpuTelemetry; memory: MemoryStatus; gpu: unknown[]; storage: unknown[]; network: unknown; batteries: unknown[]; processes: unknown[]; collector: unknown }
export interface Point { timestamp: number; cpu: number | null; memory: number | null }
export const HISTORY_LIMIT = 600;
export function pushHistory(history: Point[], snapshot: Snapshot): void {
  const previous = history.at(-1);
  if (previous?.timestamp === snapshot.timestamp_ms) return;
  history.push({ timestamp: snapshot.timestamp_ms, cpu: available(snapshot.cpu.usage), memory: ratio(snapshot.memory.used_bytes, snapshot.memory.total_bytes) });
  if (history.length > HISTORY_LIMIT) history.splice(0, history.length - HISTORY_LIMIT);
}
export function available(metric: Metric): number | null {
  return isMetric(metric) && metric.availability === 'available' && metric.quality === 'valid' && metric.age_ms <= 15000 && metric.value !== null ? metric.value : null;
}
export function ratio(used: Metric, total: Metric): number | null {
  const u = available(used), t = available(total);
  return u === null || t === null || t <= 0 || u < 0 || u > t ? null : u / t * 100;
}
export function isMetric(value: unknown): value is Metric {
  if (!value || typeof value !== 'object') return false;
  const m = value as Record<string, unknown>;
  return typeof m.id === 'string' && typeof m.kind === 'string' && typeof m.unit === 'string'
    && (m.value === null || typeof m.value === 'number' && Number.isFinite(m.value))
    && unsigned(m.timestamp_ms) && unsigned(m.age_ms)
    && boundedText(m.source) && boundedText(m.id) && boundedText(m.kind)
    && ['valid','unknown','invalid'].includes(String(m.quality))
    && ['percent','bytes','bytes_per_second','celsius','megahertz','milliseconds','watts','watt_hours','count'].includes(String(m.unit))
    && ['available','unsupported','permission_denied','temporarily_unavailable','stale','sensor_error'].includes(String(m.availability));
}
function unsigned(value: unknown): value is number { return Number.isSafeInteger(value) && (value as number) >= 0; }
function boundedText(value: unknown): value is string { return typeof value === 'string' && value.length <= 4096; }
function record(value: unknown): value is Record<string, unknown> { return !!value && typeof value === 'object' && !Array.isArray(value); }
function records(value: unknown, limit: number): value is Record<string, unknown>[] { return Array.isArray(value) && value.length <= limit && value.every(record); }
export function validateSnapshot(value: unknown): Snapshot {
  if (!value || typeof value !== 'object') throw new Error('Invalid snapshot');
  const s = value as Snapshot;
  if (!unsigned(s.timestamp_ms) || !unsigned(s.sequence) || !record(s.cpu) || !record(s.memory)
    || !unsigned(s.cpu.logical_cores) || s.cpu.logical_cores > 4096
    || ![s.cpu.usage, s.cpu.temperature_c, s.cpu.frequency_mhz, s.memory.total_bytes, s.memory.used_bytes, s.memory.available_bytes, s.memory.swap_total_bytes, s.memory.swap_used_bytes].every(isMetric)
    || !Array.isArray(s.cpu.per_core) || s.cpu.per_core.length > 4096 || !s.cpu.per_core.every(isMetric)
    || !records(s.gpu,64) || !records(s.storage,64) || !records(s.batteries,32) || !records(s.processes,100)
    || !s.storage.every(d => boundedText(d.id ?? d.name) && isMetric(d.total_bytes) && isMetric(d.available_bytes))
    || !s.processes.every(p => unsigned(p.pid) && boundedText(p.name) && typeof p.cpu_percent === 'number' && Number.isFinite(p.cpu_percent) && p.cpu_percent >= 0 && p.cpu_percent <= 100 && unsigned(p.memory_bytes)
      && ['available','unsupported','permission_denied','temporarily_unavailable','stale','sensor_error'].includes(String(p.cpu_availability)))) throw new Error('Invalid snapshot');
  return s;
}
export function safeText(value: unknown, limit = 120): string {
  return typeof value === 'string' ? value.replace(/[\u0000-\u001f\u007f]/g, '').slice(0, limit) : '—';
}
export function filterProcesses(processes: unknown[], query: string): Record<string, unknown>[] {
  return processes.filter((p): p is Record<string, unknown> => !!p && typeof p === 'object')
    .filter(p => safeText(p.name).toLocaleLowerCase().includes(query.slice(0, 120).toLocaleLowerCase())).slice(0, 50);
}
export interface Alert { id: string; state: string; title: string; observed: number; threshold: number; unit: string; duration_ms: number; evidence: string; consequence: string; recommendation: string; action_risk: string; reversible: boolean; confidence: string; acknowledged: boolean }
export interface HealthReport { state: string; alerts: Alert[]; resolved: string[]; missing_evidence: string[] }
export function validateHealthReport(value: unknown): HealthReport {
  const states=['normal','warning','critical','unknown'];
  const texts=(v:unknown)=>Array.isArray(v)&&v.length<=100&&v.every(boundedText);
  if (!record(value) || !states.includes(String(value.state)) || !records(value.alerts,100)
    || !texts(value.resolved) || !texts(value.missing_evidence)
    || !value.alerts.every(a => ['id','title','unit','evidence','consequence','recommendation','action_risk','confidence'].every(k=>boundedText(a[k]))
      && states.includes(String(a.state)) && typeof a.observed==='number' && Number.isFinite(a.observed)
      && typeof a.threshold==='number' && Number.isFinite(a.threshold) && unsigned(a.duration_ms)
      && typeof a.reversible==='boolean' && typeof a.acknowledged==='boolean')) throw new Error('Invalid diagnostics');
  return value as unknown as HealthReport;
}
