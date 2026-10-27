import { noop } from "@/lib/callbacks/noop";

/** A promise a test settles by hand, to hold what awaits it at a chosen point. */
export interface IDeferred<T> {
  readonly promise: Promise<T>;
  readonly resolve: (value: T) => void;
  readonly reject: (error: Error) => void;
}

/**
 * Creates a promise the test settles itself, for a command or call answered mid-test.
 *
 * @returns The promise and what settles it.
 */
export function deferred<T>(): IDeferred<T> {
  let resolve: (value: T) => void = noop;
  let reject: (error: Error) => void = noop;

  const promise: Promise<T> = new Promise<T>((settle, fail) => {
    resolve = settle;
    reject = fail;
  });

  return { promise, reject, resolve };
}

/**
 * Lets the microtasks already queued run, so the awaits a settled promise releases have passed.
 *
 * Ten turns cover the chains the services' flows and their callers await; a test that needs a timer advanced uses
 * Jest's fake timers instead.
 */
export async function flush(): Promise<void> {
  for (let index: number = 0; index < 10; index += 1) {
    await Promise.resolve();
  }
}
