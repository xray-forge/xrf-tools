/**
 * The value a share of the way from one to another.
 *
 * @param from - The value at none of the way.
 * @param to - The value at all of it.
 * @param share - How far between them; past either end it carries on along the same line.
 * @returns The value that far between.
 */
export function mix(from: number, to: number, share: number): number {
  return from + (to - from) * share;
}
