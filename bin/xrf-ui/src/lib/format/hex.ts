/**
 * Formats a word in hex, as a flag set, a checksum or an id is read against its source.
 *
 * @param value - The word, read as unsigned 32 bits.
 * @param digits - How many digits to pad it to, none by default.
 * @returns The word, such as `0x0000FF00`.
 */
export function formatHex(value: number, digits: number = 0): string {
  return `0x${(value >>> 0).toString(16).padStart(digits, "0").toUpperCase()}`;
}
