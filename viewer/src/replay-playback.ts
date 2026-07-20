import type { Replay } from "./wasm/backpack_battles_wasm.js";

type PlaybackControls = {
  readonly play: HTMLButtonElement;
  readonly timeline: HTMLInputElement;
  readonly tick: HTMLElement;
  readonly speed: HTMLSelectElement;
  readonly status: HTMLElement;
  readonly onTick: (index: number, animate: boolean) => void;
};

export class ReplayPlayback {
  readonly #controls: PlaybackControls;
  #replay: Replay;
  #currentTick = 0;
  #timer: number | undefined;

  constructor(controls: PlaybackControls, replay: Replay) {
    this.#controls = controls;
    this.#replay = replay;
    controls.play.addEventListener("click", () => this.#toggle());
    controls.timeline.addEventListener("input", () => {
      this.stop();
      this.show(Number.parseInt(controls.timeline.value, 10), false);
      controls.status.textContent = "Replay scrubbed.";
    });
    controls.speed.addEventListener("change", () => {
      if (this.#timer === undefined) return;
      this.stop();
      this.#start();
    });
  }

  get currentTick(): number {
    return this.#currentTick;
  }

  setReplay(replay: Replay): void {
    this.stop();
    this.#replay = replay;
    this.#controls.timeline.max = String(Math.max(replay.ticks.length - 1, 0));
    this.show(0, false);
  }

  show(index: number, animate: boolean): void {
    const safeIndex = Math.min(Math.max(index, 0), this.#replay.ticks.length - 1);
    this.#currentTick = safeIndex;
    this.#controls.timeline.value = String(safeIndex);
    this.#controls.tick.textContent = `Tick ${this.#replay.ticks[safeIndex]?.tick ?? 0}`;
    this.#controls.onTick(safeIndex, animate);
  }

  stop(): void {
    if (this.#timer !== undefined) window.clearInterval(this.#timer);
    this.#timer = undefined;
    this.#controls.play.textContent = "Play";
    this.#controls.status.textContent = "Replay paused.";
  }

  #toggle(): void {
    if (this.#timer === undefined) this.#start();
    else this.stop();
  }

  #start(): void {
    if (this.#currentTick >= this.#replay.ticks.length - 1) this.show(0, false);
    const speed = Number.parseFloat(this.#controls.speed.value);
    this.#timer = window.setInterval(() => {
      if (this.#currentTick >= this.#replay.ticks.length - 1) {
        this.stop();
        this.#controls.status.textContent = `Battle complete: ${this.#replay.outcome}.`;
        return;
      }
      this.show(this.#currentTick + 1, true);
    }, 100 / speed);
    this.#controls.play.textContent = "Pause";
    this.#controls.status.textContent = "Playing replay.";
  }
}
