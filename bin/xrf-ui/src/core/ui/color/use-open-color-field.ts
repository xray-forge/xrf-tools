import { Nullable } from "@xrf/types";
import { useCallback, useSyncExternalStore } from "react";

/** The colour field whose picker is open, application wide: opening one closes the last. */
let openField: Nullable<string> = null;
const listeners: Set<() => void> = new Set();

function subscribe(listener: () => void): () => void {
  listeners.add(listener);

  return () => listeners.delete(listener);
}

function setOpenField(field: Nullable<string>): void {
  openField = field;
  listeners.forEach((listener: () => void) => listener());
}

/**
 * @param field - The field asking, by an id unique to it.
 * @returns Whether its picker is open, and what opens or closes it, closing any other open.
 */
export function useOpenColorField(field: string): [boolean, () => void] {
  const isOpen: boolean = useSyncExternalStore(subscribe, () => openField === field);
  const toggle = useCallback(() => setOpenField(openField === field ? null : field), [field]);

  return [isOpen, toggle];
}
