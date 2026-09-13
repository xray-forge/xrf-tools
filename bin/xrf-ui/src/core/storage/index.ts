/**
 * Named one by one rather than re-exported wholesale, so `buildStorageKey` stays out of the public surface: minting a
 * key is `keys.ts`'s, which reaches it as a sibling.
 */

export * from "./keys";
export { EStorageNamespace, isInStorageNamespace } from "./namespace";
