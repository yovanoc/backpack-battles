import type { EventView, TickView } from "./wasm/backpack_battles_wasm.js";

export function summaryText(tick: TickView): string {
  const latest = tick.events.at(-1);
  const state = `Left ${tick.left_health} HP / ${tick.left_block} block · Right ${tick.right_health} HP / ${tick.right_block} block.`;
  return latest ? `${state} ${eventText(latest)}` : state;
}

function eventText(event: EventView): string {
  switch (event.kind) {
    case "activated":
      return `${event.item_kind ?? "Item"} activated.`;
    case "damage":
    case "health_lost":
      return `${event.side ?? "Hero"} lost ${event.amount ?? 0} health.`;
    case "healed":
      return `${event.side ?? "Hero"} healed ${event.amount ?? 0}.`;
    case "block":
      return `${event.side ?? "Hero"} now has ${event.amount ?? 0} block.`;
    case "speed":
      return `Item speed increased by ${event.amount ?? 0} basis points.`;
    case "fell":
      return `${event.item_kind ?? "Item"} fell.`;
    case "fall_prevented":
      return "An item prevented a fall.";
    case "consumed":
      return `${event.item_kind ?? "Item"} was consumed.`;
    case "poisoned":
      return `${event.side ?? "Hero"} has ${event.amount ?? 0} poison.`;
    case "poison_damage":
      return `${event.side ?? "Hero"} took ${event.amount ?? 0} poison damage.`;
    case "poison_cleansed":
      return `${event.side ?? "Hero"} cleansed poison; ${event.amount ?? 0} remains.`;
  }
  return "Battle event resolved.";
}

export function readInteger(
  input: HTMLInputElement,
  fallback: number,
  minimum: number,
): number {
  const value = Number.parseInt(input.value, 10);
  return Number.isFinite(value) ? Math.max(value, minimum) : fallback;
}
