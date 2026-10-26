import { useSyncExternalStore } from "react";

/**
 * How many writes this module's storage helpers have made.
 *
 * A count rather than a description of the write, because the only readers recompute everything they show anyway:
 * "something changed" is the whole signal, and a payload nobody reads is an interface nobody can rely on.
 */
let revision: number = 0;

/** One subscription shared by every caller, rather than one listener per hook instance. */
const listeners: Set<() => void> = new Set();

/**
 * Records that local storage was written.
 *
 * Deliberately absent from this directory's barrel: writing is `read-write.ts`'s, and it calls this as a sibling. A
 * surface that bumped the revision without writing would make every reader disagree with storage.
 */
export function bumpLocalStorageRevision(): void {
  revision += 1;

  for (const listener of listeners) {
    listener();
  }
}

/**
 * The current revision.
 *
 * @returns How many writes have been made through the storage helpers.
 */
export function getLocalStorageRevision(): number {
  return revision;
}

/**
 * Watches for writes made through the storage helpers.
 *
 * @param onChange - Called after each write.
 * @returns Stops watching.
 */
export function subscribeToLocalStorage(onChange: () => void): () => void {
  listeners.add(onChange);

  return () => {
    listeners.delete(onChange);
  };
}

/**
 * Re-renders when local storage is written through the storage helpers.
 *
 * Not the `storage` event, which fires on every same-origin window *except the one that wrote* - and every write here
 * is same-document, so it would never fire. That event would only earn its place alongside a second webview window,
 * and the application declares one.
 *
 * Writes that bypass the helpers are invisible to this, MUI's own `theme` key among them, so a surface that must be
 * exact reads storage again when it mounts as well.
 *
 * @returns The current revision, for a caller that wants to key work on it rather than only re-render.
 */
export function useLocalStorageRevision(): number {
  return useSyncExternalStore(subscribeToLocalStorage, getLocalStorageRevision);
}
