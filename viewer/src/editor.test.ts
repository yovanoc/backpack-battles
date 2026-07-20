import { describe, expect, test } from "bun:test";

import { placeItem, placeItemFirstAvailable, rotatedShape } from "./editor.js";
import type { ItemInfo } from "./wasm/backpack_battles_wasm.js";

const domino: ItemInfo = {
  index: 0,
  kind: "Wooden Sword",
  archetype: "aggression",
  effect: "8 normal damage",
  shape: [
    [0, 0],
    [1, 0],
  ],
  stats: {
    weapon: true,
    weight: 3,
    can_fall: true,
    armor: 0,
    max_health: 0,
    adjacent_damage: 0,
    retaliation: 0,
    vengeful: false,
    first_activation: 20,
    cadence: 20,
    natural_fall_every: undefined,
    natural_fall_one_in: undefined,
  },
};

describe("bag placement", () => {
  test("rotates and normalizes a shape", () => {
    expect(rotatedShape(domino.shape, 1)).toEqual([
      [0, 0],
      [0, 1],
    ]);
  });

  test("rejects a shape outside the bag", () => {
    expect(
      placeItem({
        placements: [],
        catalog: [domino],
        item: domino,
        x: 4,
        y: 0,
        rotation: 0,
        width: 5,
        height: 4,
      }).kind,
    ).toBe("blocked");
  });

  test("quick-add uses the first space where the item fits", () => {
    const result = placeItemFirstAvailable({
      placements: [{ kind: domino.index, x: 0, y: 0, rotation: 0 }],
      catalog: [domino],
      item: domino,
      rotation: 0,
      width: 5,
      height: 4,
    });

    expect(result).toEqual({
      kind: "placed",
      placements: [
        { kind: domino.index, x: 0, y: 0, rotation: 0 },
        { kind: domino.index, x: 2, y: 0, rotation: 0 },
      ],
    });
  });

  test("rejects a third copy of the same item", () => {
    const result = placeItem({
      placements: [
        { kind: domino.index, x: 0, y: 0, rotation: 1 },
        { kind: domino.index, x: 1, y: 0, rotation: 1 },
      ],
      catalog: [domino],
      item: domino,
      x: 2,
      y: 0,
      rotation: 1,
      width: 5,
      height: 4,
    });

    expect(result).toEqual({
      kind: "blocked",
      reason: "A bag can hold at most two copies of each item.",
    });
  });
});
