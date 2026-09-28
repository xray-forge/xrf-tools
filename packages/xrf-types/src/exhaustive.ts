/**
 * Fails to compile where a switch believed exhausted has a case left, and throws on a value no declaration holds.
 *
 * @param value - The remainder of the switch, `never` once every case is handled.
 */
export function assertExhaustive(value: never): never {
  throw new TypeError(`Unhandled case: ${JSON.stringify(value)}`);
}
