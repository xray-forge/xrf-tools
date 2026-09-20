import { Nullable } from "@/lib/types/general";

/**
 * Drops whatever text the window has selected.
 */
export function clearWindowTextSelection(): void {
  const selection: Nullable<Selection> = window.getSelection();

  // Guarded on the collapsed case rather than cleared unconditionally: removing the ranges of an empty selection is
  // harmless but fires `selectionchange`, and anything listening would hear one press as a change of nothing.
  if (selection && !selection.isCollapsed) {
    selection.removeAllRanges();
  }
}

/**
 * Clears the window's text selection whenever a pointer presses an element.
 *
 * @param element - Element whose presses clear the selection, usually a scene's canvas.
 * @returns Unbinds the listener.
 */
export function bindSelectionReset(element: HTMLElement): () => void {
  element.addEventListener("pointerdown", clearWindowTextSelection);

  return () => element.removeEventListener("pointerdown", clearWindowTextSelection);
}
