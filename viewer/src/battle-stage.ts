import {
  Application,
  Container,
  Graphics,
  Sprite,
  Text,
  Texture,
} from "pixi.js";

import { spriteUrl } from "./sprites.js";
import { itemKey, StageEffects, type ItemNode } from "./stage-effects.js";
import { stageLayout, type StageLayout } from "./stage-layout.js";
import {
  archetypeColor,
  cssColor,
  reducedMotion,
  textStyle,
} from "./stage-style.js";
import type {
  ItemInfo,
  ItemRuntimeView,
  Replay,
  SideView,
  TickView,
} from "./wasm/backpack_battles_wasm.js";

const CELL = 52;

export class BattleStage {
  readonly #app = new Application();
  readonly #textures: ReadonlyMap<string, Texture>;
  readonly #catalog: ReadonlyMap<string, ItemInfo>;
  readonly #layout: StageLayout;
  readonly #items = new Map<string, ItemNode>();
  readonly #health = new Map<SideView, Graphics>();
  readonly #block = new Map<SideView, Text>();
  readonly #overlays = new Map<SideView, Graphics>();
  readonly #effects: StageEffects;
  #replay: Replay | undefined;

  get compact(): boolean {
    return this.#layout.width === 420;
  }

  private constructor(
    textures: ReadonlyMap<string, Texture>,
    catalog: ReadonlyMap<string, ItemInfo>,
    layout: StageLayout,
  ) {
    this.#textures = textures;
    this.#catalog = catalog;
    this.#layout = layout;
    this.#effects = new StageEffects({
      app: this.#app,
      items: this.#items,
      overlays: this.#overlays,
      layout,
      cell: CELL,
    });
  }

  static async mount(
    host: HTMLElement,
    catalog: readonly ItemInfo[],
  ): Promise<BattleStage> {
    const urls = [...new Set(catalog.map(spriteUrl))];
    const entries: Array<readonly [string, Texture]> = [];
    for (const url of urls) {
      const image = new Image();
      image.src = url;
      await image.decode();
      entries.push([url, Texture.from(image)]);
    }
    const stage = new BattleStage(
      new Map(entries),
      new Map(catalog.map((item) => [item.kind, item])),
      stageLayout(host.clientWidth <= 700),
    );
    await stage.#app.init({
      width: stage.#layout.width,
      height: stage.#layout.height,
      antialias: false,
      autoDensity: true,
      background: cssColor("--surface-recessed"),
      resolution: Math.min(window.devicePixelRatio, 2),
    });
    stage.#app.canvas.setAttribute("aria-hidden", "true");
    host.replaceChildren(stage.#app.canvas);
    return stage;
  }

  destroy(): void {
    this.#effects.reset();
    this.#app.destroy(true, { children: true });
  }

  setReplay(replay: Replay): void {
    this.#effects.reset();
    this.#replay = replay;
    this.#items.clear();
    this.#health.clear();
    this.#block.clear();
    this.#overlays.clear();
    for (const child of this.#app.stage.removeChildren()) {
      child.destroy({ children: true });
    }
    this.#drawArena();
    this.#drawHero("left", replay);
    this.#drawHero("right", replay);
    this.showTick(0, false);
  }

  showTick(index: number, animate: boolean): TickView | undefined {
    const replay = this.#replay;
    if (!replay) return undefined;
    const safeIndex = Math.min(Math.max(index, 0), replay.ticks.length - 1);
    const tick = replay.ticks[safeIndex];
    if (!tick) return undefined;
    if (!animate) this.#effects.reset();

    const removed = new Set<string>();
    const removedThrough = animate ? safeIndex : safeIndex + 1;
    for (const previous of replay.ticks.slice(0, removedThrough)) {
      for (const event of previous.events) {
        if ((event.kind === "fell" || event.kind === "consumed") && event.side && event.item) {
          removed.add(itemKey(event.side, event.item));
        }
      }
    }
    for (const [key, node] of this.#items) {
      node.container.alpha = removed.has(key) ? 0.18 : 1;
      node.container.y = removed.has(key) ? 8 : 0;
      node.container.tint = "white";
    }
    this.#drawMeters("left", tick.left_health, tick.left_block, replay.left.max_health);
    this.#drawMeters("right", tick.right_health, tick.right_block, replay.right.max_health);
    this.#drawCharges("left", tick.left_items);
    this.#drawCharges("right", tick.right_items);
    if (animate && !reducedMotion()) this.#effects.animate(tick.events);
    return tick;
  }

  #drawArena(): void {
    const { width, height, bagX, bagY, versusY } = this.#layout;
    const ink = cssColor("--border-ink");
    const stone = cssColor("--surface-stone");
    const stoneLit = cssColor("--surface-stone-lit");
    this.#app.stage.addChild(
      new Graphics().rect(0, 0, width, height).fill(cssColor("--surface-recessed")),
    );
    for (const side of ["left", "right"] as const) {
      const x = bagX[side];
      const y = bagY[side];
      this.#app.stage.addChild(
        new Graphics()
          .rect(x - 10, y - 10, CELL * 5 + 20, CELL * 4 + 20)
          .fill(ink)
          .rect(x - 6, y - 6, CELL * 5 + 12, CELL * 4 + 12)
          .fill(stoneLit)
          .rect(x, y, CELL * 5, CELL * 4)
          .fill(stone),
      );
      for (let row = 0; row < 4; row += 1) {
        for (let column = 0; column < 5; column += 1) {
          this.#app.stage.addChild(
            new Graphics()
              .rect(x + column * CELL + 3, y + row * CELL + 3, CELL - 6, CELL - 6)
              .fill(cssColor("--surface-panel")),
          );
        }
      }
    }
    this.#app.stage.addChild(
      new Text({
        text: "VS",
        style: textStyle(28, cssColor("--text-primary")),
        x: width / 2 - 20,
        y: versusY,
      }),
    );
  }

  #drawHero(side: SideView, replay: Replay): void {
    const hero = side === "left" ? replay.left : replay.right;
    const x = this.#layout.bagX[side];
    const y = this.#layout.bagY[side];
    const top = this.#layout.heroTop[side];
    this.#app.stage.addChild(
      new Text({ text: hero.name, style: textStyle(18), x, y: top }),
    );
    const meter = new Graphics();
    this.#health.set(side, meter);
    this.#app.stage.addChild(meter);
    const block = new Text({ text: "", style: textStyle(14), x, y: top + 70 });
    this.#block.set(side, block);
    this.#app.stage.addChild(block);

    for (const item of hero.items) {
      const container = new Container();
      for (const [cellX, cellY] of item.cells) {
        container.addChild(
          new Graphics()
            .rect(x + cellX * CELL + 5, y + cellY * CELL + 5, CELL - 10, CELL - 10)
            .fill(archetypeColor(item.archetype)),
        );
      }
      const info = this.#catalog.get(item.kind);
      const texture = info ? this.#textures.get(spriteUrl(info)) : undefined;
      const firstCell = item.cells[0];
      if (texture && firstCell) {
        texture.source.scaleMode = "nearest";
        const sprite = new Sprite(texture);
        sprite.width = 36;
        sprite.height = 36;
        sprite.x = x + firstCell[0] * CELL + 8;
        sprite.y = y + firstCell[1] * CELL + 8;
        container.addChild(sprite);
      }
      const charge = new Graphics();
      container.addChild(charge);
      this.#items.set(itemKey(side, item.id), { container, side, charge });
      this.#app.stage.addChild(container);
    }
    const overlay = new Graphics()
      .rect(x, y, CELL * 5, CELL * 4)
      .fill("white");
    overlay.alpha = 0;
    this.#overlays.set(side, overlay);
    this.#app.stage.addChild(overlay);
  }

  #drawMeters(side: SideView, health: number, block: number, maxHealth: number): void {
    const x = this.#layout.bagX[side];
    const top = this.#layout.heroTop[side];
    const meter = this.#health.get(side);
    const blockText = this.#block.get(side);
    if (!meter || !blockText) return;
    const ratio = Math.max(0, health / maxHealth);
    meter
      .clear()
      .rect(x, top + 34, CELL * 5, 24)
      .fill(cssColor("--border-ink"))
      .rect(x + 4, top + 38, (CELL * 5 - 8) * ratio, 16)
      .fill(cssColor("--status-health"));
    blockText.text = `HP ${health}/${maxHealth}   BLOCK ${block}`;
  }

  #drawCharges(side: SideView, items: readonly ItemRuntimeView[]): void {
    for (const item of items) {
      const node = this.#items.get(itemKey(side, item.id));
      if (!node) continue;
      const progress = item.charge_progress;
      node.charge.clear();
      node.charge.visible = progress !== undefined;
      if (progress === undefined) continue;
      const x = this.#layout.bagX[side] + item.id[0] * CELL + 6;
      const y = this.#layout.bagY[side] + item.id[1] * CELL + CELL - 10;
      node.charge
        .rect(x, y, CELL - 12, 5)
        .fill(cssColor("--border-ink"))
        .rect(x + 1, y + 1, (CELL - 14) * progress, 3)
        .fill(cssColor("--status-focus"));
    }
  }

}
