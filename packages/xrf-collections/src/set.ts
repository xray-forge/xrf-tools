/**
 * A set that stays empty: `Object.freeze` guards an object's own properties, not a set's entries, so its mutators
 * refuse instead.
 */
class EmptySet extends Set<never> {
  public override add(): never {
    throw new TypeError("EMPTY_SET is shared and cannot take entries.");
  }

  public override delete(): boolean {
    return false;
  }

  public override clear(): void {}
}

/**
 * The one empty set, shared by everything that needs a stable "nothing here".
 *
 * Typed `never` so it satisfies a `ReadonlySet<T>` of any element type, and refuses entries so a caller that tries to add
 * one fails loudly rather than quietly filling in everyone else's default.
 */
export const EMPTY_SET: ReadonlySet<never> = Object.freeze(new EmptySet());
