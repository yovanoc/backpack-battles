import type { ItemInfo } from "./wasm/backpack_battles_wasm.js";

const OBJECT_TILES = [
  64, 65, 66, 67, 76, 77, 78, 79, 80, 81, 82, 83, 100, 101, 102, 103, 104,
  105, 106, 107, 112, 113, 114, 115, 116, 117, 118, 119, 124, 125, 126,
  127, 128, 129, 130, 131,
] as const;

const KEYWORD_TILES = [
  ["shield", 78],
  ["buckler", 78],
  ["targe", 79],
  ["pavise", 79],
  ["bulwark", 79],
  ["armor", 76],
  ["mail", 77],
  ["sword", 80],
  ["blade", 81],
  ["dagger", 82],
  ["rapier", 83],
  ["katana", 81],
  ["axe", 105],
  ["hammer", 104],
  ["star", 106],
  ["potion", 100],
  ["vial", 101],
  ["venom", 113],
  ["plague", 125],
  ["book", 65],
  ["grimoire", 65],
  ["bomb", 114],
  ["grenade", 114],
  ["mine", 115],
  ["trap", 116],
  ["cactus", 112],
  ["horn", 117],
  ["drum", 118],
] as const;

export function spriteTile(item: Pick<ItemInfo, "index" | "kind">): number {
  const name = item.kind.toLowerCase();
  const matched = KEYWORD_TILES.find(([keyword]) => name.includes(keyword));
  return matched?.[1] ?? OBJECT_TILES[item.index % OBJECT_TILES.length] ?? 80;
}

export function spriteUrl(item: Pick<ItemInfo, "index" | "kind">): string {
  return `/assets/tiny-dungeon/Tiles/tile_${spriteTile(item).toString().padStart(4, "0")}.png`;
}
