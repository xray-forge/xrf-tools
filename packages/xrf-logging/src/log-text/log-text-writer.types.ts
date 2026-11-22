/** Where a value inside a logged line sits while it is written. */
export interface ILogValuePlace {
  /** How many containers hold it. */
  depth: number;
  /** The containers holding it, outermost first, which a reference back to is circular. */
  ancestors: ReadonlyArray<object>;
}
