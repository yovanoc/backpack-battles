import { cellKey, itemByIndex, placementCells } from "./editor.js";
import { spriteUrl } from "./sprites.js";
import type {
  ArchetypeView,
  ItemInfo,
  Placement,
} from "./wasm/backpack_battles_wasm.js";

const ARCHETYPES: readonly (ArchetypeView | "all")[] = [
  "all",
  "aggression",
  "defense",
  "scaling",
  "control",
  "support",
];

export type PaletteRenderRequest = {
  readonly host: HTMLElement;
  readonly catalog: readonly ItemInfo[];
  readonly selected: ItemInfo;
  readonly query: string;
  readonly archetype: ArchetypeView | "all";
  readonly onSelect: (item: ItemInfo) => void;
};

export type BagRenderRequest = {
  readonly host: HTMLElement;
  readonly placements: readonly Placement[];
  readonly catalog: readonly ItemInfo[];
  readonly onCell: (x: number, y: number, placementIndex: number | undefined) => void;
};

export function renderFilters(
  host: HTMLElement,
  selected: ArchetypeView | "all",
  onSelect: (archetype: ArchetypeView | "all") => void,
): void {
  host.replaceChildren(
    ...ARCHETYPES.map((archetype) => {
      const button = document.createElement("button");
      button.type = "button";
      button.className = `filter-button ${archetype}`;
      button.textContent = archetype === "all" ? "All" : archetype;
      button.ariaPressed = String(selected === archetype);
      button.addEventListener("click", () => onSelect(archetype));
      return button;
    }),
  );
}

export function renderPalette(request: PaletteRenderRequest): void {
  const { host, catalog, selected, query, archetype, onSelect } = request;
  const normalized = query.trim().toLowerCase();
  const visible = catalog.filter(
    (item) =>
      (archetype === "all" || item.archetype === archetype) &&
      (`${item.kind} ${item.effect}`.toLowerCase().includes(normalized) || normalized === ""),
  );
  if (visible.length === 0) {
    const empty = document.createElement("p");
    empty.className = "empty-state";
    empty.textContent = "No items match that search.";
    host.replaceChildren(empty);
    return;
  }
  host.replaceChildren(
    ...visible.map((item) => {
      const button = document.createElement("button");
      button.type = "button";
      button.className = `palette-item ${item.archetype}`;
      button.ariaPressed = String(item.index === selected.index);
      button.setAttribute(
        "aria-label",
        `${item.kind}, ${item.archetype}, ${item.stats.weapon ? "weapon" : "not a weapon"}, ${item.shape.length} cells`,
      );
      const image = document.createElement("img");
      image.src = spriteUrl(item);
      image.alt = "";
      image.width = 32;
      image.height = 32;
      const copy = document.createElement("span");
      copy.innerHTML = `<strong></strong><small></small>`;
      const strong = copy.querySelector("strong");
      const small = copy.querySelector("small");
      if (strong) strong.textContent = item.kind;
      if (small) small.textContent = item.effect;
      button.append(image, copy);
      button.addEventListener("click", () => onSelect(item));
      return button;
    }),
  );
}

export function renderBag(request: BagRenderRequest): void {
  const { host, placements, catalog, onCell } = request;
  const byIndex = itemByIndex(catalog);
  const occupied = new Map<string, { readonly item: ItemInfo; readonly placementIndex: number }>();
  placements.forEach((placement, placementIndex) => {
    const item = byIndex.get(placement.kind);
    if (!item) return;
    for (const [x, y] of placementCells(placement, item)) {
      occupied.set(cellKey(x, y), { item, placementIndex });
    }
  });

  const cells: HTMLButtonElement[] = [];
  for (let y = 0; y < 4; y += 1) {
    for (let x = 0; x < 5; x += 1) {
      const occupant = occupied.get(cellKey(x, y));
      const button = document.createElement("button");
      button.type = "button";
      button.className = occupant ? `bag-cell occupied ${occupant.item.archetype}` : "bag-cell";
      button.setAttribute(
        "aria-label",
        occupant
          ? `Row ${y + 1}, column ${x + 1}: ${occupant.item.kind}. Remove item.`
          : `Row ${y + 1}, column ${x + 1}: empty. Place selected item.`,
      );
      if (occupant) {
        button.style.backgroundImage = `url("${spriteUrl(occupant.item)}")`;
        button.title = occupant.item.kind;
      }
      button.addEventListener("click", () => {
        if (!occupant || window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
          onCell(x, y, occupant?.placementIndex);
          return;
        }
        button.classList.add("removing");
        button.addEventListener(
          "animationend",
          () => onCell(x, y, occupant.placementIndex),
          { once: true },
        );
      });
      cells.push(button);
    }
  }
  host.replaceChildren(...cells);
}

export function renderInspector(host: HTMLElement, item: ItemInfo): void {
  const statRows: readonly [string, string][] = [
    ["Weapon", item.stats.weapon ? "Yes" : "No"],
    ["Weight", String(item.stats.weight)],
    ["Can fall", item.stats.can_fall ? "Yes" : "No"],
    ["Armor", String(item.stats.armor)],
    ["Max health", String(item.stats.max_health)],
    ["Adjacent damage", String(item.stats.adjacent_damage)],
    ["Retaliation", String(item.stats.retaliation)],
    ["Vengeful", item.stats.vengeful ? "Yes" : "No"],
    ["First activation", optionalTicks(item.stats.first_activation)],
    ["Cadence", optionalTicks(item.stats.cadence)],
    ["Natural fall interval", optionalTicks(item.stats.natural_fall_every)],
    ["Natural fall chance", item.stats.natural_fall_one_in ? `1 in ${item.stats.natural_fall_one_in}` : "None"],
    ["Footprint", `${item.shape.length} cells`],
  ];
  const body = document.createElement("div");
  body.className = `inspector-body ${item.archetype}`;
  const image = document.createElement("img");
  image.src = spriteUrl(item);
  image.alt = "";
  image.width = 64;
  image.height = 64;
  const heading = document.createElement("div");
  const title = document.createElement("h3");
  title.textContent = item.kind;
  const archetype = document.createElement("p");
  archetype.className = "archetype-label";
  archetype.textContent = item.archetype;
  heading.append(title, archetype);
  const effect = document.createElement("p");
  effect.className = "effect-copy";
  effect.textContent = item.effect;
  const list = document.createElement("dl");
  list.className = "stat-list";
  for (const [label, value] of statRows) {
    const term = document.createElement("dt");
    term.textContent = label;
    const detail = document.createElement("dd");
    detail.textContent = value;
    list.append(term, detail);
  }
  body.append(image, heading, effect, list);
  host.replaceChildren(body);
}

function optionalTicks(value: number | undefined): string {
  return value === undefined ? "Passive" : `${value} ticks`;
}
