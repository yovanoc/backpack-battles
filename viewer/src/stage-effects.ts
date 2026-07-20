import { Application, Container, Graphics, Text, type Ticker } from "pixi.js";

import { cssColor, eventLabel, textStyle } from "./stage-style.js";
import type { StageLayout } from "./stage-layout.js";
import { startDamageTracer } from "./stage-tracer.js";
import type {
  EventView,
  FallCauseView,
  SideView,
} from "./wasm/backpack_battles_wasm.js";

export type ItemNode = {
  readonly container: Container;
  readonly side: SideView;
  readonly charge: Graphics;
};

type StageEffectsConfig = {
  readonly app: Application;
  readonly items: ReadonlyMap<string, ItemNode>;
  readonly overlays: ReadonlyMap<SideView, Graphics>;
  readonly layout: StageLayout;
  readonly cell: number;
};

type ActiveLabel = {
  readonly container: Container;
  readonly cancel: () => void;
};

type LabelFragment = { readonly text: string; readonly color: string };

export class StageEffects {
  readonly #config: StageEffectsConfig;
  readonly #labels = new Map<SideView, ActiveLabel>();
  readonly #tweens = new Set<() => void>();

  constructor(config: StageEffectsConfig) {
    this.#config = config;
  }

  animate(events: readonly EventView[]): void {
    const labels = new Map<SideView, LabelFragment[]>();
    const impacts = new Set<SideView>();
    const heals = new Set<SideView>();
    for (const event of events) {
      if (event.kind === "activated" && event.side && event.item) {
        const node = this.#config.items.get(itemKey(event.side, event.item));
        if (node) this.#flash(node.container);
      }
      const label = eventLabel(event);
      if (label && event.side) {
        const existing = labels.get(event.side) ?? [];
        existing.push(label);
        labels.set(event.side, existing);
      }
      if ((event.kind === "fell" || event.kind === "consumed") && event.side && event.item) {
        const node = this.#config.items.get(itemKey(event.side, event.item));
        if (node) this.#fadeItem(node.container, event.side, event.cause);
      }
      if ((event.kind === "poisoned" || event.kind === "poison_damage") && event.side) {
        this.#poisonFlash(event.side);
      }
      if ((event.kind === "damage" || event.kind === "poison_damage") && event.side) {
        impacts.add(event.side);
      }
      if (event.kind === "damage" && event.side && event.item) {
        let cancel = (): void => {};
        cancel = startDamageTracer({
          app: this.#config.app,
          layout: this.#config.layout,
          cell: this.#config.cell,
          targetSide: event.side,
          sourceCell: event.item,
          mode: event.mode,
          done: () => this.#tweens.delete(cancel),
        });
        this.#tweens.add(cancel);
      }
      if (event.kind === "healed" && event.side) heals.add(event.side);
      if (event.kind === "healed" && event.side && event.item) {
        this.#healSource(event.side, event.item);
      }
      if (event.kind === "block" && event.side) {
        this.#shock(event.side, cssColor("--status-block"));
      }
      if (event.kind === "fall_prevented" && event.side) {
        this.#shock(event.side, cssColor("--status-focus"));
      }
      if (event.kind === "poison_cleansed" && event.side) {
        this.#shock(event.side, cssColor("--archetype-scaling"));
      }
    }
    for (const [side, fragments] of labels) this.#floatLabel(side, fragments);
    for (const side of impacts) this.#impact(side);
    for (const side of heals) this.#healGlow(side);
  }

  reset(): void {
    for (const cancel of [...this.#tweens]) cancel();
    for (const active of this.#labels.values()) active.container.destroy({ children: true });
    this.#labels.clear();
    for (const node of this.#config.items.values()) {
      node.container.alpha = 1;
      node.container.x = 0;
      node.container.y = 0;
      node.container.tint = "white";
    }
    for (const overlay of this.#config.overlays.values()) {
      overlay.alpha = 0;
      overlay.tint = "white";
    }
  }

  #flash(container: Container, color = cssColor("--status-focus")): void {
    container.alpha = 0.35;
    container.tint = color;
    this.#tween(
      420,
      (progress) => {
        container.alpha = 0.35 + progress * 0.65;
      },
      () => {
        container.tint = "white";
      },
    );
  }

  #fadeItem(container: Container, side: SideView, cause: FallCauseView | undefined): void {
    this.#tween(720, (progress) => {
      container.alpha = 1 - progress * 0.82;
      container.y = progress * (cause === "forced" ? 4 : 18);
      container.x = cause === "forced" ? progress * (side === "left" ? -32 : 32) : 0;
    });
  }

  #poisonFlash(side: SideView): void {
    const nodes = [...this.#config.items.values()].filter((node) => node.side === side);
    for (const node of nodes) node.container.tint = cssColor("--status-poison");
    this.#shock(side, cssColor("--status-poison"));
    this.#tween(
      720,
      () => {},
      () => {
        for (const node of nodes) node.container.tint = "white";
      },
    );
  }

  #impact(side: SideView): void {
    const nodes = [...this.#config.items.values()].filter((node) => node.side === side);
    this.#shock(side, cssColor("--archetype-aggression"));
    this.#tween(
      420,
      (progress) => {
        const offset = Math.sin(progress * Math.PI * 8) * (1 - progress) * 12;
        for (const node of nodes) node.container.x = offset;
      },
      () => {
        for (const node of nodes) node.container.x = 0;
      },
    );
  }

  #healGlow(side: SideView): void {
    this.#shock(side, cssColor("--archetype-scaling"));
  }

  #healSource(side: SideView, item: readonly [number, number]): void {
    const node = this.#config.items.get(itemKey(side, item));
    if (node) this.#flash(node.container, cssColor("--archetype-scaling"));
  }

  #shock(side: SideView, color: string): void {
    const overlay = this.#config.overlays.get(side);
    if (!overlay) return;
    overlay.tint = color;
    this.#tween(
      720,
      (progress) => {
        overlay.alpha = Math.sin(progress * Math.PI) * 0.5;
      },
      () => {
        overlay.alpha = 0;
        overlay.tint = "white";
      },
    );
  }

  #floatLabel(side: SideView, fragments: readonly LabelFragment[]): void {
    const { app, layout, cell } = this.#config;
    const active = this.#labels.get(side);
    active?.cancel();
    active?.container.destroy({ children: true });
    const container = new Container({
      x: layout.bagX[side] + cell * 2,
      y: layout.feedbackY[side],
    });
    let offset = 0;
    const labels: Text[] = [];
    for (const [index, fragment] of fragments.entries()) {
      const label = new Text({
        text: `${index === 0 ? "" : "  "}${fragment.text}`,
        style: textStyle(22, fragment.color),
        x: offset,
      });
      offset += label.width;
      labels.push(label);
    }
    const backing = new Graphics()
      .roundRect(-8, -5, offset + 16, 34, 4)
      .fill(cssColor("--surface-canvas"))
      .stroke({ color: cssColor("--border-ink"), width: 3 });
    container.addChild(backing, ...labels);
    app.stage.addChild(container);
    const cancel = this.#tween(
      720,
      (progress) => {
        container.y = layout.feedbackY[side] - progress * 20;
        container.alpha = 1 - progress;
      },
      () => {
        container.destroy({ children: true });
        this.#labels.delete(side);
      },
    );
    this.#labels.set(side, { container, cancel });
  }

  #tween(
    duration: number,
    update: (progress: number) => void,
    done?: () => void,
  ): () => void {
    let elapsed = 0;
    let running = true;
    const cancel = (): void => {
      if (!running) return;
      running = false;
      this.#config.app.ticker.remove(tick);
      this.#tweens.delete(cancel);
    };
    const tick = (ticker: Ticker): void => {
      elapsed += ticker.deltaMS;
      const progress = Math.min(elapsed / duration, 1);
      update(progress);
      if (progress === 1) {
        cancel();
        done?.();
      }
    };
    this.#config.app.ticker.add(tick);
    this.#tweens.add(cancel);
    return cancel;
  }
}

export function itemKey(side: SideView, cell: readonly [number, number]): string {
  return `${side}:${cell[0]}:${cell[1]}`;
}
