import "./style.css";

import { renderBag, renderFilters, renderInspector, renderPalette } from "./bag-view.js";
import { BattleStage } from "./battle-stage.js";
import { renderCombatView } from "./combat-view.js";
import { bindEditorDialog } from "./editor-dialog.js";
import {
  inferPlacements,
  placeItem,
  placeItemFirstAvailable,
  removePlacement,
} from "./editor.js";
import {
  defaultHealth,
  initEngine,
  listItems,
  runBattle,
  runBattleWithBags,
} from "./engine.js";
import { readInteger, summaryText } from "./replay-text.js";
import { ReplayPlayback } from "./replay-playback.js";
import { mountView } from "./view.js";
import type {
  ArchetypeView,
  Placement,
  Replay,
  SideView,
} from "./wasm/backpack_battles_wasm.js";

class ViewerStateError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "ViewerStateError";
  }
}

const root = document.getElementById("app");
if (!(root instanceof HTMLElement)) throw new ViewerStateError("Missing #app root");

await initEngine();
const catalog = listItems();
const firstItem = catalog[0];
if (!firstItem) throw new ViewerStateError("The item catalog is empty");

const view = mountView(root);
view.health.value = String(defaultHealth());
let stage = await BattleStage.mount(view.stage, catalog);

let selectedItem = firstItem;
let selectedArchetype: ArchetypeView | "all" = "all";
let catalogView: "list" | "grid" = "list";
let rotation = 0;
let leftPlacements: readonly Placement[] = [];
let rightPlacements: readonly Placement[] = [];
let replay = runGeneratedBattle();
const playback = new ReplayPlayback(
  {
    play: view.play,
    timeline: view.timeline,
    tick: view.tick,
    speed: view.speed,
    status: view.status,
    onTick: showTick,
  },
  replay,
);

loadReplay(replay, true);
renderEditor();
bindEditorDialog({
  dialog: view.editorDialog,
  open: view.openEditor,
  close: view.closeEditor,
  fight: view.fight,
  fightFromEditor: view.fightFromEditor,
});

view.search.addEventListener("input", renderPaletteView);
view.listView.addEventListener("click", () => setCatalogView("list"));
view.gridView.addEventListener("click", () => setCatalogView("grid"));
view.addLeft.addEventListener("click", () => quickAdd("left"));
view.addRight.addEventListener("click", () => quickAdd("right"));
view.generate.addEventListener("click", () => {
  playback.stop();
  loadReplay(runGeneratedBattle(), true);
  view.status.textContent = "Generated deterministic bags.";
});
view.fight.addEventListener("click", () => {
  playback.stop();
  try {
    const next = runBattleWithBags(
      leftPlacements,
      rightPlacements,
      readInteger(view.seed, 42, 0),
      readInteger(view.health, defaultHealth(), 1),
      readInteger(view.tickLimit, 2000, 1),
    );
    loadReplay(next, false);
    view.status.textContent = "Edited bags resolved.";
  } catch (error) {
    view.status.textContent = error instanceof Error ? error.message : String(error);
  }
});
view.rotate.addEventListener("click", () => {
  rotation = (rotation + 1) % 4;
  view.rotation.textContent = `${rotation * 90}°`;
  view.status.textContent = `Placement rotated to ${rotation * 90}°.`;
});
view.clearLeft.addEventListener("click", () => {
  leftPlacements = [];
  renderBags();
  view.status.textContent = "Left bag cleared.";
});
view.clearRight.addEventListener("click", () => {
  rightPlacements = [];
  renderBags();
  view.status.textContent = "Right bag cleared.";
});
let remounting = false;
new ResizeObserver(() => {
  const compact = view.stage.clientWidth <= 700;
  if (compact === stage.compact || remounting) return;
  remounting = true;
  playback.stop();
  const previous = stage;
  void BattleStage.mount(view.stage, catalog)
    .then((next) => {
      stage = next;
      previous.destroy();
      stage.setReplay(replay);
      playback.show(playback.currentTick, false);
    })
    .catch((error: unknown) => {
      view.status.textContent = error instanceof Error ? error.message : String(error);
    })
    .finally(() => {
      remounting = false;
    });
}).observe(view.stage);

function runGeneratedBattle(): Replay {
  return runBattle(
    readInteger(view.seed, 42, 0),
    readInteger(view.health, defaultHealth(), 1),
    readInteger(view.tickLimit, 2000, 1),
  );
}

function loadReplay(next: Replay, syncEditor: boolean): void {
  replay = next;
  if (syncEditor) {
    leftPlacements = inferPlacements(next.left, catalog);
    rightPlacements = inferPlacements(next.right, catalog);
    renderBags();
  }
  stage.setReplay(next);
  playback.setReplay(next);
}

function renderEditor(): void {
  renderFilters(view.filters, selectedArchetype, (archetype) => {
    selectedArchetype = archetype;
    renderEditor();
  });
  renderPaletteView();
  renderInspector(view.inspectorBody, selectedItem);
  renderBags();
}

function renderPaletteView(): void {
  view.palette.classList.toggle("grid-view", catalogView === "grid");
  renderPalette({
    host: view.palette,
    catalog,
    selected: selectedItem,
    query: view.search.value,
    archetype: selectedArchetype,
    onSelect: (item) => {
      selectedItem = item;
      renderPaletteView();
      renderInspector(view.inspectorBody, item);
    },
  });
}

function setCatalogView(next: "list" | "grid"): void {
  catalogView = next;
  view.listView.ariaPressed = String(next === "list");
  view.gridView.ariaPressed = String(next === "grid");
  renderPaletteView();
}

function renderBags(): void {
  renderBag({
    host: view.leftBag,
    placements: leftPlacements,
    catalog,
    onCell: (x, y, placementIndex) => editCell("left", x, y, placementIndex),
  });
  renderBag({
    host: view.rightBag,
    placements: rightPlacements,
    catalog,
    onCell: (x, y, placementIndex) => editCell("right", x, y, placementIndex),
  });
}

function editCell(
  side: SideView,
  x: number,
  y: number,
  placementIndex: number | undefined,
): void {
  const placements = side === "left" ? leftPlacements : rightPlacements;
  if (placementIndex !== undefined) {
    setPlacements(side, removePlacement(placements, placementIndex));
    view.status.textContent = "Item removed.";
    return;
  }
  const result = placeItem({
    placements,
    catalog,
    item: selectedItem,
    x,
    y,
    rotation,
    width: replay.bag_width,
    height: replay.bag_height,
  });
  if (result.kind === "blocked") {
    view.status.textContent = result.reason;
    return;
  }
  setPlacements(side, result.placements);
  view.status.textContent = `${selectedItem.kind} placed on ${side}.`;
}

function quickAdd(side: SideView): void {
  const placements = side === "left" ? leftPlacements : rightPlacements;
  const result = placeItemFirstAvailable({
    placements,
    catalog,
    item: selectedItem,
    rotation,
    width: replay.bag_width,
    height: replay.bag_height,
  });
  if (result.kind === "blocked") {
    view.status.textContent = result.reason;
    return;
  }
  setPlacements(side, result.placements);
  view.status.textContent = `${selectedItem.kind} added to ${side}.`;
}

function setPlacements(side: SideView, placements: readonly Placement[]): void {
  if (side === "left") leftPlacements = placements;
  else rightPlacements = placements;
  renderBags();
}

function showTick(index: number, animate: boolean): void {
  const tick = stage.showTick(index, animate);
  if (!tick) return;
  view.summary.textContent = summaryText(tick);
  renderCombatView(view.effects, view.fightLog, replay, index);
}
