import { Nullable } from "@xrf/types";

/**
 * A renderer contract type with every float settled to a number. The generated contract spells an `f32` as
 * `number | null`, since a NaN crosses as null; a value the application holds and sends is never one.
 */
export type TSettled<T> = [T] extends [Nullable<number>]
  ? [T] extends [null]
    ? T
    : number
  : T extends ReadonlyArray<unknown>
    ? { readonly [K in keyof T]: TSettled<T[K]> }
    : T extends object
      ? { [K in keyof T]: TSettled<T[K]> }
      : T;
