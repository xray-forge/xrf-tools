import { RefObject, useEffect } from "react";

import { Nullable } from "@/lib/types/general";

/**
 * Puts the caret in a search field from anywhere on the page.
 *
 * `/` only reaches the field while nothing else has the caret, because inside a field it is a character
 * someone is typing rather than a shortcut.
 *
 * todo: Should have dedicated keybind core domain and service, encapsulate this one into it.
 *
 * @param inputRef - Field the shortcut focuses and selects.
 */
export function useSearchHotkey(inputRef: RefObject<Nullable<HTMLInputElement>>): void {
  useEffect(() => {
    function onWindowKeyDown(event: KeyboardEvent): void {
      const isEditing: boolean =
        event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement;

      if (((event.ctrlKey || event.metaKey) && event.key === "k") || (event.key === "/" && !isEditing)) {
        event.preventDefault();
        inputRef.current?.focus();
        inputRef.current?.select();
      }
    }

    window.addEventListener("keydown", onWindowKeyDown);

    return () => window.removeEventListener("keydown", onWindowKeyDown);
  }, [inputRef]);
}
