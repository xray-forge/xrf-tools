/**
 * Whether a value is a plain record read by key: an object, neither null nor an array. What a parser of stored or
 * hand-written JSON asks before reading a group's keys.
 *
 * @param value - Any value.
 * @returns Whether it is such a record.
 */
export function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
