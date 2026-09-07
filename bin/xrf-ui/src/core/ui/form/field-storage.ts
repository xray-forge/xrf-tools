/**
 * Where a form field's own memory lives.
 *
 * One module rather than a prefix spelled in each hook, because a surface reporting on what the application has
 * stored has to be able to ask which keys are whose, and a string repeated in three places is not an answer.
 */

/** Holds the one value a field currently has. */
const VALUE_PREFIX: string = "xrf.form.";

/** Holds the short history of values a field was given. */
const RECENTS_PREFIX: string = "xrf.form-recents.";

/**
 * Where one field's current value is stored.
 *
 * @param application - Application the field belongs to.
 * @param id - Field name within that application.
 * @returns The storage key.
 */
export function getFieldValueStorageKey(application: string, id: string): string {
  return `${VALUE_PREFIX}${application}.${id}`;
}

/**
 * Where one field's history is stored.
 *
 * @param application - Application the field belongs to.
 * @param id - Field name within that application.
 * @returns The storage key.
 */
export function getFieldRecentsStorageKey(application: string, id: string): string {
  return `${RECENTS_PREFIX}${application}.${id}`;
}

/**
 * Whether a key holds a field's current value.
 *
 * @param key - Storage key to classify.
 * @returns Whether it belongs to a field's value.
 */
export function isFieldValueStorageKey(key: string): boolean {
  return key.startsWith(VALUE_PREFIX);
}

/**
 * Whether a key holds a field's history.
 *
 * @param key - Storage key to classify.
 * @returns Whether it belongs to a field's history.
 */
export function isFieldRecentsStorageKey(key: string): boolean {
  return key.startsWith(RECENTS_PREFIX);
}
