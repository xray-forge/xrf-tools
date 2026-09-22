import { CommandType } from "@wirestate/core";
import { useOnCommand } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { RefObject } from "react";

/**  Asks the search field that is on screen for the caret. */
export const FOCUS_SEARCH_FIELD_MESSAGE: CommandType = Symbol("@/search/focus-field");

/**
 * Makes a field the one the caret goes to, for as long as it is mounted.
 *
 * Registration is the whole state: `CommandBus.hasHandler` answers whether a field is on screen, and the handler
 * stack makes the newest mounted field the one that answers.
 *
 * @param inputRef - Field to focus and select.
 */
export function useSearchFocusTarget(inputRef: RefObject<Nullable<HTMLInputElement>>): void {
  useOnCommand(FOCUS_SEARCH_FIELD_MESSAGE, () => {
    inputRef.current?.focus();
    inputRef.current?.select();
  });
}
