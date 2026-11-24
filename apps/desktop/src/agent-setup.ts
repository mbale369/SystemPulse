import {invoke} from '@tauri-apps/api/core';
import {demoMode} from './backend';
import {el} from './view';
import {mcpConfiguration} from './widget-model';
import type {Language} from './i18n';
export interface AgentPreparation {enabled:boolean;client:'codex'|'claude'}
export function renderAgentSetup(language:Language,state:AgentPreparation):HTMLElement {
  const fr=language==='fr',box=el('section','agent-setup');
  box.append(el('h3','',fr?'Agents · connexion contrôlée':'Agents · controlled setup'));
  const label=el('label','agent-switch'),toggle=el('input');toggle.type='checkbox';toggle.checked=state.enabled;
  toggle.setAttribute('role','switch');toggle.setAttribute('aria-label',fr?'Préparer la connexion MCP':'Prepare MCP connection');
  label.append(toggle,el('span','',fr?'Préparer la connexion MCP':'Prepare MCP connection'));box.append(label);
  const status=el('p','agent-status'),details=el('div','agent-details');
  const select=el('select');select.setAttribute('aria-label',fr?'Agent cible':'Target agent');
  for(const [value,text] of [['codex','Codex · TOML'],['claude','Claude · JSON']]){const option=el('option','',text);option.value=value!;select.append(option);}
  select.value=state.client;
  const config=el('textarea');config.readOnly=true;config.rows=5;config.spellcheck=false;
  config.setAttribute('aria-label',fr?'Configuration MCP à copier':'MCP configuration to copy');
  details.append(select,config,el('p','',fr?'Copiez cette configuration dans le client de votre choix après l’avoir relue. Aucun fichier Claude/Codex n’est modifié.':'Review and copy this configuration into your chosen client. No Claude/Codex file is modified.'));
  box.append(status,details,el('p','agent-privacy',fr?'Lecture seule, sans noms de processus par défaut. Une fois configuré, le client peut envoyer les mesures à son fournisseur IA. Ce switch ne révoque pas une configuration déjà copiée : désactivez-la aussi dans le client.':'Read-only, no process names by default. Once configured, your client may send metrics to its AI provider. This switch does not revoke copied configuration: disable it in the client too.'));
  let command=demoMode?'/installation/systempulse-mcp':'';
  let available=demoMode;
  const update=()=>{
    details.hidden=!state.enabled;toggle.checked=state.enabled;toggle.disabled=!available;
    status.textContent=!available?(fr?'Binaire MCP absent de cette installation.':'MCP binary missing from this installation.'):
      state.enabled?(fr?'Prêt à configurer · connexion externe non vérifiée':'Ready to configure · external connection not verified'):
      (fr?'Aucune connexion créée par SystemPulse.':'No connection created by SystemPulse.');
    if(state.enabled&&available)config.value=mcpConfiguration(state.client,command);
  };
  toggle.addEventListener('change',()=>{
    if(toggle.checked&&!window.confirm(fr?'Afficher une configuration MCP locale en lecture seule ? Aucun agent ne sera connecté automatiquement.':'Show a local read-only MCP configuration? No agent will be connected automatically.')){toggle.checked=false;return;}
    state.enabled=toggle.checked;update();
  });
  select.addEventListener('change',()=>{state.client=select.value==='claude'?'claude':'codex';update();});
  update();
  if(!demoMode)void invoke<{command:string;available:boolean}>('get_mcp_launch_info').then(info=>{
    if(typeof info.command!=='string'||typeof info.available!=='boolean')throw new Error('Invalid setup');
    if(info.available)mcpConfiguration('codex',info.command);
    command=info.command;available=info.available;update();
  }).catch(()=>{available=false;update();});
  return box;
}
