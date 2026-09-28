/**
 * @param count - How many processes.
 * @returns The count with its noun, such as `1 process` or `6 processes`.
 */
export function describeProcessCount(count: number): string {
  return count === 1 ? "1 process" : `${count} processes`;
}
