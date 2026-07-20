export type ViewElements = {
  readonly seed: HTMLInputElement;
  readonly health: HTMLInputElement;
  readonly tickLimit: HTMLInputElement;
  readonly generate: HTMLButtonElement;
  readonly fight: HTMLButtonElement;
  readonly editorDialog: HTMLDialogElement;
  readonly openEditor: HTMLButtonElement;
  readonly closeEditor: HTMLButtonElement;
  readonly fightFromEditor: HTMLButtonElement;
  readonly search: HTMLInputElement;
  readonly filters: HTMLElement;
  readonly palette: HTMLElement;
  readonly listView: HTMLButtonElement;
  readonly gridView: HTMLButtonElement;
  readonly inspectorBody: HTMLElement;
  readonly addLeft: HTMLButtonElement;
  readonly addRight: HTMLButtonElement;
  readonly stage: HTMLElement;
  readonly effects: HTMLElement;
  readonly fightLog: HTMLOListElement;
  readonly summary: HTMLElement;
  readonly status: HTMLElement;
  readonly leftBag: HTMLElement;
  readonly rightBag: HTMLElement;
  readonly rotate: HTMLButtonElement;
  readonly rotation: HTMLElement;
  readonly clearLeft: HTMLButtonElement;
  readonly clearRight: HTMLButtonElement;
  readonly play: HTMLButtonElement;
  readonly timeline: HTMLInputElement;
  readonly tick: HTMLElement;
  readonly speed: HTMLSelectElement;
};

export class MissingViewElementError extends Error {
  constructor(selector: string) {
    super(`Missing viewer element: ${selector}`);
    this.name = "MissingViewElementError";
  }
}

export function mountView(root: HTMLElement): ViewElements {
  root.innerHTML = `
    <header class="app-header">
      <div>
        <p class="eyebrow">DETERMINISTIC BATTLE WORKBENCH</p>
        <h1>Backpack Battles</h1>
        <p class="subtitle">Edit both bags, inspect every authored stat, then replay the exact fight.</p>
      </div>
      <div class="run-controls" aria-label="Battle setup">
        <label class="field">Seed<input id="seed" type="number" min="0" step="1" value="42"></label>
        <label class="field">Health<input id="health" type="number" min="1" max="65535" value="250"></label>
        <label class="field">Tick limit<input id="tick-limit" type="number" min="1" max="2000" value="2000"></label>
        <button id="generate" class="pixel-button secondary" type="button">Generate bags</button>
        <button id="fight" class="pixel-button primary" type="button">Fight edited bags</button>
      </div>
    </header>

    <main class="workbench">
      <section class="pixel-panel arena-panel" aria-labelledby="arena-title">
        <div class="panel-heading arena-heading">
          <div><p class="eyebrow">LIVE WASM ENGINE</p><h2 id="arena-title">Battle arena</h2></div>
          <div class="arena-actions">
            <div id="status" class="status-text" role="status">Ready</div>
            <button id="open-editor" class="pixel-button secondary compact" type="button">Edit loadouts</button>
          </div>
        </div>
        <div class="arena-visuals">
          <div id="stage" class="battle-stage"></div>
          <aside class="combat-rail" aria-label="Live battle details">
            <section aria-labelledby="effects-title"><h3 id="effects-title">Current effects</h3><div id="effects" class="effects-panel"></div></section>
            <section class="log-section" aria-labelledby="fight-log-title"><h3 id="fight-log-title">Fight log</h3><ol id="fight-log" class="fight-log"></ol></section>
          </aside>
        </div>
        <div id="summary" class="battle-summary" aria-live="polite">Battle ready.</div>
        <div class="timeline" aria-label="Replay controls">
          <button id="play" class="pixel-button primary compact" type="button">Play</button>
          <label class="timeline-slider">Battle tick<input id="timeline" type="range" min="0" max="0" value="0"></label>
          <output id="tick" for="timeline">Tick 0</output>
          <label class="field compact-field">Speed<select id="speed"><option value="0.5">0.5×</option><option value="1" selected>1×</option><option value="2">2×</option><option value="4">4×</option></select></label>
        </div>

      </section>
    </main>

    <dialog id="editor-dialog" class="editor-dialog" aria-labelledby="editor-dialog-title">
      <div class="editor-dialog-shell">
        <header class="editor-dialog-header">
          <div><p class="eyebrow">LOADOUT WORKSHOP</p><h2 id="editor-dialog-title">Items & bags</h2></div>
          <button id="close-editor" class="pixel-button secondary compact" type="button">Close</button>
        </header>
        <div class="editor-workspace">
          <aside class="pixel-panel palette-panel" aria-labelledby="palette-title">
            <div class="panel-heading"><div><p class="eyebrow">52 ITEMS</p><h2 id="palette-title">Item palette</h2></div></div>
            <label class="field search-field">Find item<input id="search" type="search" placeholder="Sword, poison, shield…"></label>
            <div id="filters" class="filter-row" aria-label="Filter by archetype"></div>
            <div class="catalog-view-toggle" aria-label="Catalog layout">
              <button id="list-view" class="filter-button" type="button" aria-pressed="true">List</button>
              <button id="grid-view" class="filter-button" type="button" aria-pressed="false">Grid</button>
            </div>
            <div id="palette" class="palette-list"></div>
          </aside>

          <section class="pixel-panel loadout-panel" aria-labelledby="loadout-title">
            <div class="panel-heading"><div><p class="eyebrow">MAX TWO OF EACH ITEM</p><h2 id="loadout-title">Arrange bags</h2></div></div>
            <div class="editor-toolbar">
              <span class="placement-label">Place selected item</span>
              <button id="rotate" class="pixel-button secondary compact" type="button">Rotate</button>
              <span id="rotation" class="rotation-label">0°</span>
            </div>
            <div class="bag-editors">
              <section aria-labelledby="left-bag-title">
                <div class="bag-heading"><h3 id="left-bag-title">Left bag</h3><button id="clear-left" class="text-button" type="button">Clear</button></div>
                <div id="left-bag" class="bag-grid" aria-label="Left hero bag"></div>
              </section>
              <section aria-labelledby="right-bag-title">
                <div class="bag-heading"><h3 id="right-bag-title">Right bag</h3><button id="clear-right" class="text-button" type="button">Clear</button></div>
                <div id="right-bag" class="bag-grid" aria-label="Right hero bag"></div>
              </section>
            </div>
            <p class="editor-help">Choose an item, then select an empty anchor cell. Select an occupied cell to remove it.</p>
          </section>

          <aside id="inspector" class="pixel-panel inspector-panel" aria-labelledby="inspector-title">
            <div class="panel-heading"><div><p class="eyebrow">AUTHORED DATA</p><h2 id="inspector-title">Item stats</h2></div></div>
            <div id="inspector-body"></div>
            <div class="quick-add" aria-label="Quick add selected item">
              <button id="add-left" class="pixel-button secondary compact" type="button">Add left</button>
              <button id="add-right" class="pixel-button secondary compact" type="button">Add right</button>
            </div>
          </aside>
        </div>
        <footer class="editor-dialog-footer">
          <p>Changes stay local until you start the edited fight.</p>
          <button id="fight-from-editor" class="pixel-button primary" type="button">Close & fight</button>
        </footer>
      </div>
    </dialog>
  `;

  return {
    seed: required(root.querySelector<HTMLInputElement>("#seed"), "#seed"),
    health: required(root.querySelector<HTMLInputElement>("#health"), "#health"),
    tickLimit: required(root.querySelector<HTMLInputElement>("#tick-limit"), "#tick-limit"),
    generate: required(root.querySelector<HTMLButtonElement>("#generate"), "#generate"),
    fight: required(root.querySelector<HTMLButtonElement>("#fight"), "#fight"),
    editorDialog: required(root.querySelector<HTMLDialogElement>("#editor-dialog"), "#editor-dialog"),
    openEditor: required(root.querySelector<HTMLButtonElement>("#open-editor"), "#open-editor"),
    closeEditor: required(root.querySelector<HTMLButtonElement>("#close-editor"), "#close-editor"),
    fightFromEditor: required(root.querySelector<HTMLButtonElement>("#fight-from-editor"), "#fight-from-editor"),
    search: required(root.querySelector<HTMLInputElement>("#search"), "#search"),
    filters: required(root.querySelector<HTMLElement>("#filters"), "#filters"),
    palette: required(root.querySelector<HTMLElement>("#palette"), "#palette"),
    listView: required(root.querySelector<HTMLButtonElement>("#list-view"), "#list-view"),
    gridView: required(root.querySelector<HTMLButtonElement>("#grid-view"), "#grid-view"),
    inspectorBody: required(root.querySelector<HTMLElement>("#inspector-body"), "#inspector-body"),
    addLeft: required(root.querySelector<HTMLButtonElement>("#add-left"), "#add-left"),
    addRight: required(root.querySelector<HTMLButtonElement>("#add-right"), "#add-right"),
    stage: required(root.querySelector<HTMLElement>("#stage"), "#stage"),
    effects: required(root.querySelector<HTMLElement>("#effects"), "#effects"),
    fightLog: required(root.querySelector<HTMLOListElement>("#fight-log"), "#fight-log"),
    summary: required(root.querySelector<HTMLElement>("#summary"), "#summary"),
    status: required(root.querySelector<HTMLElement>("#status"), "#status"),
    leftBag: required(root.querySelector<HTMLElement>("#left-bag"), "#left-bag"),
    rightBag: required(root.querySelector<HTMLElement>("#right-bag"), "#right-bag"),
    rotate: required(root.querySelector<HTMLButtonElement>("#rotate"), "#rotate"),
    rotation: required(root.querySelector<HTMLElement>("#rotation"), "#rotation"),
    clearLeft: required(root.querySelector<HTMLButtonElement>("#clear-left"), "#clear-left"),
    clearRight: required(root.querySelector<HTMLButtonElement>("#clear-right"), "#clear-right"),
    play: required(root.querySelector<HTMLButtonElement>("#play"), "#play"),
    timeline: required(root.querySelector<HTMLInputElement>("#timeline"), "#timeline"),
    tick: required(root.querySelector<HTMLElement>("#tick"), "#tick"),
    speed: required(root.querySelector<HTMLSelectElement>("#speed"), "#speed"),
  };
}

function required<T>(value: T | null, selector: string): T {
  if (value === null) throw new MissingViewElementError(selector);
  return value;
}
