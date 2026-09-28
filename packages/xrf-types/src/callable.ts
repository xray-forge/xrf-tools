/** A function of these arguments returning this. */
export type Callable<T extends Array<unknown> = Array<unknown>, R = void> = (...args: T) => R;
