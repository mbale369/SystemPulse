export function soundEnabled(storage?:Pick<Storage,'getItem'>):boolean{
  try{return storage?.getItem('sound-enabled')!=='off';}catch{return true;}
}
export function saveSound(enabled:boolean,storage?:Pick<Storage,'setItem'>){try{storage?.setItem('sound-enabled',enabled?'on':'off');}catch{/* session-only */}}
export class SoundCooldown {
  private last:number|null=null;
  accept(now:number):boolean{if(!Number.isFinite(now)||now<0)return false;if(this.last!==null&&now<this.last){this.last=now;return false;}if(this.last!==null&&now-this.last<30000)return false;this.last=now;return true;}
}
export class SubtleAudio {
  private context:AudioContext|null=null;
  private gate=new SoundCooldown();
  private active:OscillatorNode|null=null;
  isReady(){return this.context?.state==='running';}
  async unlock():Promise<boolean>{
    try{this.context??=new AudioContext();if(this.context.state==='suspended')await this.context.resume();return this.context.state==='running';}catch{return false;}
  }
  play(enabled:boolean,now:number,test=false):boolean{
    const ctx=this.context;
    if(!enabled||!ctx||ctx.state!=='running'||(!test&&!this.gate.accept(now))||this.active)return false;
    const oscillator=ctx.createOscillator(),gain=ctx.createGain(),start=ctx.currentTime;
    oscillator.type='sine';oscillator.frequency.value=660;
    gain.gain.setValueAtTime(0,start);gain.gain.linearRampToValueAtTime(.02,start+.02);gain.gain.linearRampToValueAtTime(0,start+.14);
    oscillator.connect(gain);gain.connect(ctx.destination);this.active=oscillator;
    oscillator.onended=()=>{oscillator.disconnect();gain.disconnect();this.active=null;};
    oscillator.start(start);oscillator.stop(start+.15);return true;
  }
  mute(){try{this.active?.stop();}catch{/* already ended */}}
  close(){this.mute();void this.context?.close().catch(()=>{});}
}
