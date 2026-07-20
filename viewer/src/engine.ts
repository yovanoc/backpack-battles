// Typed wrapper around the wasm bridge. Initialise once, then call the
// deterministic engine. All heavy DTOs come back as plain JS objects.
import init, {
  default_health,
  list_items,
  run_battle,
  run_battle_with_bags,
} from "./wasm/backpack_battles_wasm.js";
import type {
  ItemInfo,
  Placement,
  Replay,
} from "./wasm/backpack_battles_wasm.js";

let ready: Promise<void> | null = null;

/** Load and instantiate the wasm module (idempotent). */
export function initEngine(): Promise<void> {
  if (!ready) {
    ready = init().then(() => undefined);
  }
  return ready;
}

/** Base hero health the engine uses when `health` is 0. */
export function defaultHealth(): number {
  return default_health();
}

/** Every item kind with stats + footprint, for the palette and legend. */
export function listItems(): ItemInfo[] {
  return [...list_items()];
}

/** Run a battle from the two seed-generated demo bags. */
export function runBattle(
  seed: number | bigint,
  health: number,
  ticks: number,
): Replay {
  return run_battle(BigInt(seed), health, ticks);
}

/** Run a battle from two caller-built bags (the editor path). */
export function runBattleWithBags(
  left: readonly Placement[],
  right: readonly Placement[],
  seed: number | bigint,
  health: number,
  ticks: number,
): Replay {
  return run_battle_with_bags([...left], [...right], BigInt(seed), health, ticks);
}
