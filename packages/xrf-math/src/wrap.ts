/**
 * The value counted round a period from zero, as a time of day or a bearing is.
 *
 * @param value - The value, any number of periods either way.
 * @param period - How long one round is.
 * @returns Where in the round the value falls, in `[0, period)` for a positive period.
 */
export function wrap(value: number, period: number): number {
  return ((value % period) + period) % period;
}
