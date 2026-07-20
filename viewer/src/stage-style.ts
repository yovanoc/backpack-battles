import type { TextStyleOptions } from "pixi.js";

import type {
  ArchetypeView,
  EventView,
} from "./wasm/backpack_battles_wasm.js";

export function eventLabel(
  event: EventView,
): { readonly text: string; readonly color: string } | undefined {
  if (!event.amount) return undefined;
  if (event.kind === "healed") {
    return { text: `+${event.amount}`, color: cssColor("--archetype-scaling") };
  }
  if (event.kind === "block") {
    return { text: `BLOCK ${event.amount}`, color: cssColor("--status-block") };
  }
  if (event.kind === "poisoned") {
    return { text: `POISON ${event.amount}`, color: cssColor("--status-poison") };
  }
  if (
    event.kind === "damage" ||
    event.kind === "health_lost" ||
    event.kind === "poison_damage"
  ) {
    return { text: `-${event.amount}`, color: cssColor("--archetype-aggression") };
  }
  return undefined;
}

export function textStyle(
  fontSize: number,
  fill = cssColor("--text-primary"),
): TextStyleOptions {
  return { fill, fontFamily: "monospace", fontSize, fontWeight: "700" };
}

export function archetypeColor(archetype: ArchetypeView): string {
  return cssColor(`--archetype-${archetype}`);
}

export function cssColor(token: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(token).trim();
}

export function reducedMotion(): boolean {
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}
