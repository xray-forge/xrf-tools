import { Nullable } from "@xrf/types";

import { isModalOpen } from "@/lib/dom/modal";
import { isTextEntryTarget } from "@/lib/dom/text-entry";

/**
 * Whether a viewport the pointer enters may take the keyboard focus, so its keys reach it without a click first.
 *
 * Not while another application is active, text is being entered, a dialog or a menu is open, or a button is held,
 * which is a drag under way somewhere.
 *
 * @param pointer - The pointer entering the viewport.
 * @returns Whether the viewport may focus itself.
 */
export function canFocusViewportOnHover(pointer: PointerEvent): boolean {
  const active: Nullable<Element> = document.activeElement;

  return (
    document.hasFocus() &&
    pointer.buttons === 0 &&
    !isTextEntryTarget(active) &&
    !(active instanceof HTMLSelectElement) &&
    // A listbox does not count: code views and choice lists stay on screen as listboxes, and a popup's own listbox
    // keeps the focus in its text field.
    !isModalOpen()
  );
}
