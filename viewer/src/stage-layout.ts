import type { SideView } from "./wasm/backpack_battles_wasm.js";

export type StageLayout = {
  readonly width: number;
  readonly height: number;
  readonly bagX: Readonly<Record<SideView, number>>;
  readonly bagY: Readonly<Record<SideView, number>>;
  readonly heroTop: Readonly<Record<SideView, number>>;
  readonly feedbackY: Readonly<Record<SideView, number>>;
  readonly versusY: number;
};

export function stageLayout(compact: boolean): StageLayout {
  if (compact) {
    return {
      width: 420,
      height: 760,
      bagX: { left: 80, right: 80 },
      bagY: { left: 110, right: 470 },
      heroTop: { left: 18, right: 380 },
      feedbackY: { left: 326, right: 698 },
      versusY: 344,
    };
  }
  return {
    width: 920,
    height: 430,
    bagX: { left: 58, right: 602 },
    bagY: { left: 118, right: 118 },
    heroTop: { left: 18, right: 18 },
    feedbackY: { left: 364, right: 364 },
    versusY: 212,
  };
}
