import { isRecord } from "@xrf/types";

/**
 * Values at dotted keys, one a nested group's path (`shadows.contact.steps`), as a file a line a value holds them.
 */
export type TDottedRecord<T = unknown> = Record<string, T>;

/**
 * @param value - Values in nested groups, or a single value.
 * @param prefix - The dotted key they stand under, empty for none.
 * @returns Every value at its dotted key; a run of values, being one value, is not split.
 */
export function flattenDottedRecord(value: unknown, prefix: string = ""): TDottedRecord {
  if (!isRecord(value)) {
    return value === undefined ? {} : { [prefix]: value };
  }

  const flat: TDottedRecord = {};

  for (const [key, child] of Object.entries(value)) {
    Object.assign(flat, flattenDottedRecord(child, prefix ? `${prefix}.${key}` : key));
  }

  return flat;
}

/**
 * @param flat - Values at dotted keys.
 * @returns The same values in nested groups, by their keys' parts.
 */
export function unflattenDottedRecord(flat: Readonly<TDottedRecord>): Record<string, unknown> {
  const nested: Record<string, unknown> = {};

  for (const [key, value] of Object.entries(flat)) {
    const parts: Array<string> = key.split(".");
    let group: Record<string, unknown> = nested;

    for (const part of parts.slice(0, -1)) {
      group[part] = isRecord(group[part]) ? group[part] : {};
      group = group[part] as Record<string, unknown>;
    }

    group[parts[parts.length - 1]] = value;
  }

  return nested;
}
