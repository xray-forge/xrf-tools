import { AnyObject } from "@xrf/types";

/**
 * Formats a spawn-row value for single-line display.
 *
 * @param value - Value from the selected spawn row.
 * @returns Readable text for the value, including nested values.
 */
export function formatSpawnRowDetailsValue(value: unknown): string {
  if (value === null || value === undefined) {
    return "-";
  }

  if (Array.isArray(value)) {
    return value.length ? value.map(formatSpawnRowDetailsValue).join(", ") : "empty";
  }

  if (typeof value === "object") {
    const entries: Array<[string, unknown]> = Object.entries(value as AnyObject);

    return entries
      .map(([key, nested]: [string, unknown]) => `${key}: ${formatSpawnRowDetailsValue(nested)}`)
      .join(", ");
  }

  return String(value);
}
