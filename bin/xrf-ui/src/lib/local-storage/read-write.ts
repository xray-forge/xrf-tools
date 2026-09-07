import { bumpLocalStorageRevision } from "@/lib/local-storage/revision";
import { Logger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

/**
 * Reads a raw local storage value.
 *
 * @param key - Storage key to read.
 * @returns The stored value, or `null` when unavailable or absent.
 */
export function getLocalStorageValue(key: string): Nullable<string> {
  return window.localStorage ? window.localStorage.getItem(key) : null;
}

/**
 * Stores a raw value, or removes the key when the value is `null`.
 *
 * @param key - Storage key to write.
 * @param value - Value to persist, or `null` to clear the key.
 */
export function setLocalStorageValue(key: string, value: Nullable<string>): void {
  if (window.localStorage) {
    Logger.debug("Write local storage value:", key);
  } else {
    Logger.warn("Local storage is not available during write:", key);

    return;
  }

  if (value === null) {
    window.localStorage.removeItem(key);
  } else {
    window.localStorage.setItem(key, value);
  }

  bumpLocalStorageRevision();
}

/**
 * Parses a JSON value from local storage.
 *
 * Invalid JSON propagates the `SyntaxError` thrown by `JSON.parse`.
 *
 * @param key - Storage key to read.
 * @returns The parsed value, or `null` when storage is unavailable or the key is absent.
 */
export function parseLocalStorageValue<T>(key: string): Nullable<T> {
  if (!window.localStorage) {
    return null;
  }

  const raw: Nullable<string> = window.localStorage.getItem(key) ?? null;

  return raw === null ? null : JSON.parse(raw);
}

/**
 * Parses a JSON value that the application can do without.
 *
 * The counterpart to {@link parseLocalStorageValue} for data that is a convenience rather than configuration: unusable
 * content reads as absent instead of throwing. Deliberately not the default, because a key carrying something the
 * application needs is better read loudly than quietly discarded.
 *
 * @param key - Storage key to read.
 * @returns The parsed value, or `null` when storage is unavailable, the key is absent, or the content does not parse.
 */
export function parseLocalStorageValueSafe<T>(key: string): Nullable<T> {
  try {
    return parseLocalStorageValue<T>(key);
  } catch (error: unknown) {
    Logger.warn("Discarding unreadable local storage value:", key, error);

    return null;
  }
}

/**
 * Stores a value the application can do without, reporting failure rather than raising it.
 *
 * Writes fail for reasons a caller of this kind cannot act on - an exhausted quota, a mode that refuses storage - and
 * the work that asked for the write is worth finishing anyway.
 *
 * @param key - Storage key to write.
 * @param value - Value to persist, or `null` to clear the key.
 */
export function setLocalStorageValueSafe(key: string, value: Nullable<string>): void {
  try {
    setLocalStorageValue(key, value);
  } catch (error: unknown) {
    // Reported rather than answered: a caller of this kind has nothing to do about an exhausted quota, and the
    // Settings storage section is where a person sees the size and clears it.
    Logger.warn("Failed to write local storage value:", key, error);
  }
}
