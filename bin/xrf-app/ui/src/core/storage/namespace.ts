/**
 * The shape every local storage key this application writes has.
 *
 * `xrf.<namespace>.<name>`: a dot separates levels, and a level that needs more than one word is spelled in kebab case,
 * as an application id already is (`configs-verifier`). Dots rather than dashes throughout, because a dash cannot also
 * be the separator without making `xrf-form-recents-configs-verifier-directory` one unparseable run.
 */

/** The level every key carries, and the one this application answers to. */
const ROOT: string = "xrf";

export enum EStorageNamespace {
  /** The one value a form field is currently holding. */
  FORM = "form",
  /** The short history of values a form field was given. */
  FORM_RECENTS = "form-recents",
  /** Which side panels are open, and how wide they are. */
  PANELS = "panels",
  /** Application wide switches, one value each. */
  PREFERENCE = "preference",
}

/**
 * Deliberately absent from this directory's barrel: `keys.ts` reaches it as a sibling, and it is the inventory's one
 * constructor. Exported further, any caller could mint a key of its own.
 *
 * @param namespace - Namespace the key belongs to.
 * @param segments - Levels below the namespace, outermost first.
 * @returns The storage key.
 */
export function buildStorageKey(namespace: EStorageNamespace, ...segments: ReadonlyArray<string>): string {
  return [ROOT, namespace, ...segments].join(".");
}

/**
 * Tested with the trailing dot, so a namespace claims only what is nested under it: `form` does not answer for
 * `form-recents`, and would not for a `formatting` added later either.
 *
 * @param key - Storage key to classify.
 * @param namespace - Namespace to test against.
 * @returns Whether the key is nested under that namespace.
 */
export function isInStorageNamespace(key: string, namespace: EStorageNamespace): boolean {
  return key.startsWith(`${ROOT}.${namespace}.`);
}
