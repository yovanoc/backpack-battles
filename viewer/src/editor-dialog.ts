type EditorDialogElements = {
  readonly dialog: HTMLDialogElement;
  readonly open: HTMLButtonElement;
  readonly close: HTMLButtonElement;
  readonly fight: HTMLButtonElement;
  readonly fightFromEditor: HTMLButtonElement;
};

export function bindEditorDialog(elements: EditorDialogElements): void {
  elements.open.addEventListener("click", () => {
    elements.dialog.showModal();
    elements.dialog.querySelector<HTMLElement>(".editor-workspace")?.scrollTo(0, 0);
    elements.close.focus();
  });
  elements.close.addEventListener("click", () => elements.dialog.close());
  elements.fightFromEditor.addEventListener("click", () => {
    elements.dialog.close();
    elements.fight.click();
  });
  elements.dialog.addEventListener("click", (event) => {
    if (event.target === elements.dialog) elements.dialog.close();
  });
  elements.dialog.addEventListener("close", () => elements.open.focus());
}
