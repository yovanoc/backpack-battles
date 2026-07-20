import type {
  HeroView,
  ItemInfo,
  Placement,
} from "./wasm/backpack_battles_wasm.js";

type Cell = readonly [number, number];

export type PlacementResult =
  | { readonly kind: "placed"; readonly placements: readonly Placement[] }
  | { readonly kind: "blocked"; readonly reason: string };

export type PlaceRequest = {
  readonly placements: readonly Placement[];
  readonly catalog: readonly ItemInfo[];
  readonly item: ItemInfo;
  readonly x: number;
  readonly y: number;
  readonly rotation: number;
  readonly width: number;
  readonly height: number;
};

export function rotatedShape(
  shape: readonly Cell[],
  rotation: number,
): readonly Cell[] {
  const turns = ((rotation % 4) + 4) % 4;
  const rotated = shape.map(([x, y]): Cell => {
    switch (turns) {
      case 0:
        return [x, y];
      case 1:
        return [-y, x];
      case 2:
        return [-x, -y];
      case 3:
        return [y, -x];
      default:
        return [x, y];
    }
  });
  const minX = Math.min(...rotated.map(([x]) => x));
  const minY = Math.min(...rotated.map(([, y]) => y));
  return rotated.map(([x, y]): Cell => [x - minX, y - minY]);
}

export function placeItem(request: PlaceRequest): PlacementResult {
  const { placements, catalog, item, x, y, rotation, width, height } = request;
  if (hasTwoCopies(placements, item.index)) {
    return { kind: "blocked", reason: "A bag can hold at most two copies of each item." };
  }
  const cells = rotatedShape(item.shape, rotation).map(
    ([offsetX, offsetY]): Cell => [x + offsetX, y + offsetY],
  );
  if (cells.some(([cellX, cellY]) => cellX >= width || cellY >= height)) {
    return { kind: "blocked", reason: "That shape does not fit there." };
  }
  const occupied = occupiedKeys(placements, itemByIndex(catalog));
  if (cells.some(([cellX, cellY]) => occupied.has(cellKey(cellX, cellY)))) {
    return { kind: "blocked", reason: "That space is already occupied." };
  }
  return {
    kind: "placed",
    placements: [...placements, { kind: item.index, x, y, rotation }],
  };
}

export function placeItemFirstAvailable(
  request: Omit<PlaceRequest, "x" | "y">,
): PlacementResult {
  if (hasTwoCopies(request.placements, request.item.index)) {
    return { kind: "blocked", reason: "A bag can hold at most two copies of each item." };
  }
  for (let y = 0; y < request.height; y += 1) {
    for (let x = 0; x < request.width; x += 1) {
      const result = placeItem({ ...request, x, y });
      if (result.kind === "placed") return result;
    }
  }
  return { kind: "blocked", reason: "No space remains for that item." };
}

function hasTwoCopies(placements: readonly Placement[], kind: number): boolean {
  return placements.filter((placement) => placement.kind === kind).length >= 2;
}

export function removePlacement(
  placements: readonly Placement[],
  index: number,
): readonly Placement[] {
  return placements.filter((_, placementIndex) => placementIndex !== index);
}

export function placementCells(
  placement: Placement,
  item: ItemInfo,
): readonly Cell[] {
  return rotatedShape(item.shape, placement.rotation ?? 0).map(
    ([x, y]): Cell => [placement.x + x, placement.y + y],
  );
}

export function inferPlacements(
  hero: HeroView,
  catalog: readonly ItemInfo[],
): readonly Placement[] {
  const byName = new Map(catalog.map((item) => [item.kind, item]));
  return hero.items.flatMap((item) => {
    const info = byName.get(item.kind);
    if (!info) return [];
    const minX = Math.min(...item.cells.map(([x]) => x));
    const minY = Math.min(...item.cells.map(([, y]) => y));
    const relative = normalizedKey(
      item.cells.map(([x, y]): Cell => [x - minX, y - minY]),
    );
    const rotation = [0, 1, 2, 3].find(
      (turn) => normalizedKey(rotatedShape(info.shape, turn)) === relative,
    );
    return [{ kind: info.index, x: minX, y: minY, rotation: rotation ?? 0 }];
  });
}

export function itemByIndex(
  catalog: readonly ItemInfo[],
): ReadonlyMap<number, ItemInfo> {
  return new Map(catalog.map((item) => [item.index, item]));
}

function occupiedKeys(
  placements: readonly Placement[],
  catalog: ReadonlyMap<number, ItemInfo>,
): ReadonlySet<string> {
  return new Set(
    placements.flatMap((placement) => {
      const item = catalog.get(placement.kind);
      return item
        ? placementCells(placement, item).map(([x, y]) => cellKey(x, y))
        : [];
    }),
  );
}

export function cellKey(x: number, y: number): string {
  return `${x}:${y}`;
}

function normalizedKey(cells: readonly Cell[]): string {
  return [...cells]
    .sort(([leftX, leftY], [rightX, rightY]) => leftY - rightY || leftX - rightX)
    .map(([x, y]) => `${x}:${y}`)
    .join("|");
}
