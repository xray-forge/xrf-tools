export * from "./read-write";

/**
 * Named one by one rather than re-exported wholesale, so `bumpLocalStorageRevision` stays out of the public surface:
 * recording a write is `read-write.ts`'s, which reaches it as a sibling.
 */
export { getLocalStorageRevision, subscribeToLocalStorage, useLocalStorageRevision } from "./revision";
