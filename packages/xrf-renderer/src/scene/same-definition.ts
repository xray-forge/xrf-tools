/**
 * Whether two definitions a consumer handed over are the same: equal primitives, arrays and typed arrays of the same
 * values, plain objects of the same keys holding the same. A definition crosses the worker's boundary as a copy, so a
 * consumer sending the same one again sends an equal one, never the one held.
 *
 * @param a - One definition.
 * @param b - The other.
 * @returns Whether they are equal throughout.
 */
export function isSameDefinition(a: unknown, b: unknown): boolean {
  if (a === b) {
    return true;
  }

  if (typeof a !== "object" || typeof b !== "object" || a === null || b === null) {
    return false;
  }

  if (ArrayBuffer.isView(a) || ArrayBuffer.isView(b) || Array.isArray(a) || Array.isArray(b)) {
    const left = a as ArrayLike<unknown>;
    const right = b as ArrayLike<unknown>;

    if (Array.isArray(a) !== Array.isArray(b) || left.length !== right.length) {
      return false;
    }

    for (let index: number = 0; index < left.length; index += 1) {
      if (left[index] !== right[index] && !isSameDefinition(left[index], right[index])) {
        return false;
      }
    }

    return true;
  }

  const left = a as Record<string, unknown>;
  const right = b as Record<string, unknown>;
  const keys: Array<string> = Object.keys(left);

  return (
    keys.length === Object.keys(right).length &&
    keys.every((key: string) => Object.hasOwn(right, key) && isSameDefinition(left[key], right[key]))
  );
}
