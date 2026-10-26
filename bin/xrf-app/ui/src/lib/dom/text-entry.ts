import { Maybe } from "@xrf/types";

/**
 * Input types that carry no text, so a bare character reaching one of them is a shortcut rather than typing.
 */
const NON_TEXT_INPUT_TYPES: ReadonlySet<string> = new Set([
  "button",
  "checkbox",
  "color",
  "file",
  "image",
  "radio",
  "range",
  "reset",
  "submit",
]);

/**
 * Whether a key event's target is somewhere a person is entering text.
 *
 * @param target - Event target to classify.
 * @returns Whether text entry has the key.
 */
export function isTextEntryTarget(target: Maybe<EventTarget>): boolean {
  if (!(target instanceof HTMLElement)) {
    return false;
  }

  if (target.isContentEditable || target instanceof HTMLTextAreaElement) {
    return true;
  }

  return target instanceof HTMLInputElement && !NON_TEXT_INPUT_TYPES.has(target.type);
}
