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
 * The one empty set, shared wherever a stable "nothing here" is needed.
 *
 * Typed `never` so it fits a `ReadonlySet<T>` of any element type. Adding to it throws, since it is everyone's default.
 */
export const EMPTY_SET: ReadonlySet<never> = Object.freeze(new EmptySet());
