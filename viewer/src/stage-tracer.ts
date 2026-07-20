import { Application, Graphics, type Ticker } from "pixi.js";

import { cssColor } from "./stage-style.js";
import type { StageLayout } from "./stage-layout.js";
import type { DamageModeView, SideView } from "./wasm/backpack_battles_wasm.js";

type DamageTracerRequest = {
  readonly app: Application;
  readonly layout: StageLayout;
  readonly cell: number;
  readonly targetSide: SideView;
  readonly sourceCell: readonly [number, number];
  readonly mode: DamageModeView | undefined;
  readonly done: () => void;
};

export function startDamageTracer(request: DamageTracerRequest): () => void {
  const sourceSide: SideView = request.targetSide === "left" ? "right" : "left";
  const fromX = request.layout.bagX[sourceSide] + (request.sourceCell[0] + 0.5) * request.cell;
  const fromY = request.layout.bagY[sourceSide] + (request.sourceCell[1] + 0.5) * request.cell;
  const toX = request.layout.bagX[request.targetSide] + request.cell * 2.5;
  const toY = request.layout.bagY[request.targetSide] + request.cell * 2;
  const color = request.mode === "piercing"
    ? cssColor("--status-focus")
    : request.mode === "retaliation"
      ? cssColor("--status-block")
      : cssColor("--archetype-aggression");
  const tracer = new Graphics();
  let elapsed = 0;
  let running = true;
  const cancel = (): void => {
    if (!running) return;
    running = false;
    request.app.ticker.remove(tick);
    tracer.destroy();
    request.done();
  };
  const tick = (ticker: Ticker): void => {
    elapsed += ticker.deltaMS;
    const progress = Math.min(elapsed / 720, 1);
    const x = fromX + (toX - fromX) * progress;
    const y = fromY + (toY - fromY) * progress;
    tracer
      .clear()
      .moveTo(fromX, fromY)
      .lineTo(x, y)
      .stroke({ color, width: 10 })
      .rect(x - 7, y - 7, 14, 14)
      .fill(color);
    tracer.alpha = 1 - progress * 0.65;
    if (progress === 1) {
      cancel();
    }
  };
  request.app.stage.addChild(tracer);
  request.app.ticker.add(tick);
  return cancel;
}
