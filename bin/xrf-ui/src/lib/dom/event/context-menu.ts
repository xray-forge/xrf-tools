import { Nullable } from "@/lib/types/general";

/**
 * Targets whose own menu is the only clipboard UI they have. Matched by attribute rather than by the
 * `isContentEditable` flag so an ancestor editable region counts and jsdom can exercise it.
 */
const EDITABLE_SELECTOR: string = "input, textarea, [contenteditable]:not([contenteditable='false'])";

/**
 * Checks whether a context menu request came from an editable field.
 *
 * @param target - Target of the context menu event.
 * @returns Whether the target sits in an editable field.
 */
function isEditableTarget(target: Nullable<EventTarget>): boolean {
  return Boolean(target instanceof Element && target.closest(EDITABLE_SELECTOR));
}

/**
 * Suppresses the webview's own context menu so the application can own that gesture.
 *
 * @returns Disposer that restores the webview menu.
 */
export function suppressNativeContextMenu(): () => void {
  function onContextMenu(event: MouseEvent): void {
    if (!isEditableTarget(event.target)) {
      event.preventDefault();
    }
  }

  document.addEventListener("contextmenu", onContextMenu);

  return () => document.removeEventListener("contextmenu", onContextMenu);
}
