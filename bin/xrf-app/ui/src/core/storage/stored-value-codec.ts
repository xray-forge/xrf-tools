import { Nullable } from "@xrf/types";

/**
 * How one stored value is kept as a string: the value's whole format, defaults included, in one place.
 */
export interface IStoredValueCodec<T> {
  /**
   * @param stored - What the key holds, or `null` when it holds nothing.
   * @returns The value, the default where nothing usable is stored; throwing counts as nothing stored.
   */
  read(stored: Nullable<string>): T;
  /**
   * @param value - Value to keep.
   * @returns What the key holds from now on, or `null` to clear it.
   */
  write(value: T): Nullable<string>;
}

/**
 * Keeps a value as its own text.
 *
 * @param read - Reads what the key holds, `null` when nothing is.
 * @param write - Writes the value, as `String` does by default.
 * @returns The codec.
 */
export function textCodec<T>(
  read: (stored: Nullable<string>) => T,
  write: (value: T) => Nullable<string> = String
): IStoredValueCodec<T> {
  return { read, write };
}

/**
 * Keeps a value as JSON, a `null` value clearing the key.
 *
 * @param read - Reads the parsed value, `null` when nothing is stored.
 * @param write - Writes the value, as JSON by default.
 * @returns The codec.
 */
export function jsonCodec<T>(
  read: (parsed: unknown) => T,
  write: (value: T) => Nullable<string> = (value: T) => (value === null ? null : JSON.stringify(value))
): IStoredValueCodec<T> {
  return { read: (stored: Nullable<string>) => read(stored === null ? null : JSON.parse(stored)), write };
}

/**
 * Keeps a flag as `true` or `false`.
 *
 * @param fallback - The flag while neither is stored.
 * @returns The codec.
 */
export function booleanCodec(fallback: boolean): IStoredValueCodec<boolean> {
  return textCodec((stored: Nullable<string>) => (stored === String(!fallback) ? !fallback : fallback));
}
