export type AnyObject = Record<string, any>;

export type Optional<T> = T | undefined;

export type Nullable<T> = T | null;

export type Maybe<T> = T | null | undefined;

export type Callable<T extends Array<unknown> = Array<unknown>, R = void> = (...args: T) => R;

export type AnyCallable<T = void> = (...args: Array<unknown>) => T;
