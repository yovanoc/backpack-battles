import type {
  HeroView,
  ItemRuntimeView,
  Replay,
  SideView,
  TickView,
} from "./wasm/backpack_battles_wasm.js";

export function renderCombatView(
  effects: HTMLElement,
  fightLog: HTMLOListElement,
  replay: Replay,
  tickIndex: number,
): void {
  const tick = replay.ticks[tickIndex];
  if (!tick) return;
  effects.replaceChildren(
    heroStatus("left", replay.left, tick.left_items, tick.left_block, tick.left_poison),
    heroStatus("right", replay.right, tick.right_items, tick.right_block, tick.right_poison),
  );
  const rows = replay.ticks
    .slice(0, tickIndex + 1)
    .flatMap((current) => current.events.map((event) => ({ event, tick: current.tick })))
    .slice(-80)
    .map(({ event, tick: eventTick }) => {
      const row = document.createElement("li");
      row.textContent = `${eventTick}. ${eventText(event.kind, event.side, event.item_kind, event.amount)}`;
      return row;
    });
  if (rows.length === 0) {
    const empty = document.createElement("li");
    empty.className = "empty-state";
    empty.textContent = "No effects resolved yet.";
    rows.push(empty);
  }
  fightLog.replaceChildren(...rows);
  fightLog.scrollTop = fightLog.scrollHeight;
}

function heroStatus(
  side: SideView,
  hero: HeroView,
  runtime: readonly ItemRuntimeView[],
  block: number,
  poison: number,
): HTMLElement {
  const section = document.createElement("section");
  section.className = `hero-effects ${side}`;
  const heading = document.createElement("h4");
  heading.textContent = hero.name;
  const reserves = document.createElement("p");
  reserves.className = "effect-reserves";
  reserves.textContent = `Block ${block} · Poison ${poison}`;
  const items = new Map(hero.items.map((item) => [`${item.id[0]}:${item.id[1]}`, item]));
  const charges = runtime.flatMap((item) => {
    if (item.charge_progress === undefined) return [];
    const authored = items.get(`${item.id[0]}:${item.id[1]}`);
    if (!authored) return [];
    const row = document.createElement("label");
    row.className = "charge-row";
    const copy = document.createElement("span");
    const speed = 1 + item.speed_basis_points / 10_000;
    copy.textContent = `${authored.kind} · ${authored.stats.cadence ?? "?"}t · ${speed.toFixed(2)}×`;
    const progress = document.createElement("progress");
    progress.max = 1;
    progress.value = item.charge_progress;
    row.append(copy, progress);
    return [row];
  });
  section.append(heading, reserves, ...charges);
  return section;
}

function eventText(
  kind: TickView["events"][number]["kind"],
  side: SideView | undefined,
  itemKind: string | undefined,
  amount: number | undefined,
): string {
  const owner = side ? `${side} ` : "";
  const item = itemKind ? `${itemKind} ` : "";
  const value = amount === undefined ? "" : ` ${amount}`;
  return `${owner}${item}${kind.replaceAll("_", " ")}${value}`.trim();
}
