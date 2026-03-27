// Simple sound manager using Web Audio API to avoid external assets
class SoundManager {
  private ctx: AudioContext | null = null;

  private init() {
    if (!this.ctx) {
      this.ctx = new (window.AudioContext || (window as any).webkitAudioContext)();
    }
    if (this.ctx.state === "suspended") {
      void this.ctx.resume();
    }
  }

  private playTone(freq: number, duration: number, type: OscillatorType = "sine", volume = 0.1) {
    this.init();
    if (!this.ctx) return;

    const osc = this.ctx.createOscillator();
    const gain = this.ctx.createGain();

    osc.type = type;
    osc.frequency.setValueAtTime(freq, this.ctx.currentTime);

    gain.gain.setValueAtTime(volume, this.ctx.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.00001, this.ctx.currentTime + duration);

    osc.connect(gain);
    gain.connect(this.ctx.destination);

    osc.start();
    osc.stop(this.ctx.currentTime + duration);
  }

  playClick() {
    this.playTone(800, 0.1, "sine", 0.05);
  }

  playSuccess() {
    this.playTone(523.25, 0.1, "sine", 0.1); // C5
    setTimeout(() => this.playTone(659.25, 0.2, "sine", 0.1), 100); // E5
  }

  playError() {
    this.playTone(392.0, 0.1, "sawtooth", 0.1); // G4
    setTimeout(() => this.playTone(349.23, 0.2, "sawtooth", 0.1), 100); // F4
  }

  playSwitch() {
    this.playTone(1000, 0.05, "sine", 0.03);
  }
}

export const sounds = new SoundManager();
