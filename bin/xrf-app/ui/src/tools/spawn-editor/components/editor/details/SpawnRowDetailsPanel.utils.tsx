import { AnyObject } from "@xrf/types";

/**
 * Formats a spawn-row value for single-line display.
 *
 * A nested list or object is bracketed, so its members never read as the outer value's: `links: [[1, 1], [2, 1]]`.
 *
 * @param value - Value from the selected spawn row.
 * @returns Readable text for the value, including nested values.
 */
export function formatSpawnRowDetailsValue(value: unknown): string {
  return formatValue(value, false);
}

function formatValue(value: unknown, isNested: boolean): string {
  if (value === null || value === undefined) {
    return "-";
  }

  if (Array.isArray(value)) {
    if (!value.length) {
      return "empty";
    }

    const items: string = value.map((item: unknown) => formatValue(item, true)).join(", ");

    return isNested ? `[${items}]` : items;
  }

  if (typeof value === "object") {
    const fields: string = Object.entries(value as AnyObject)
      .map(([key, nested]: [string, unknown]) => `${key}: ${formatValue(nested, true)}`)
      .join(", ");

    return isNested ? `{ ${fields} }` : fields;
  }

  return String(value);
}
