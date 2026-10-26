export * from "./keys";
export * from "./read-write";

/**
 * Named one by one rather than re-exported wholesale, so `buildStorageKey` and `bumpLocalStorageRevision` stay out of
 * the public surface: minting a key is `keys.ts`'s and recording a write is `read-write.ts`'s, each reaching its helper
 * as a sibling.
 */
export { EStorageNamespace, isInStorageNamespace } from "./namespace";
export { getLocalStorageRevision, subscribeToLocalStorage, useLocalStorageRevision } from "./revision";
