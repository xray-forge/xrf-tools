import { TFlow } from "@/lib/mobx/flow/types";

/**
 * Awaits several promises inside a flow at once, keeping each one's type.
 *
 * What `Promise.all` is to `await`, this is to {@link call}: the reads go out together rather than one after the
 * other, and the results come back as a tuple rather than an array of unions.
 *
 * One suspension point, so a cancellation lands either before all of them or after all of them - the same granularity
 * `Promise.all` gives, and the reason independent reads belong in one of these rather than in a chain of `call`s.
 *
 * @param promises - Promises to await together.
 * @returns Their resolved values, positionally typed.
 */
export function* all<T extends ReadonlyArray<unknown>>(
  promises: readonly [...T]
): TFlow<{ -readonly [K in keyof T]: Awaited<T[K]> }> {
  return yield* call(Promise.all(promises));
}

/**
 * Awaits a promise inside a flow while keeping its type.
 *
 * A bare `yield` is typed `any`, because a generator cannot describe what is sent back into it. Delegating with
 * `yield*` through this does describe it, so `const bake: VisualMotionBake = yield* call(openMotion(name))` checks
 * exactly as the `await` it replaces did.
 *
 * @param promise - Promise to await inside the flow.
 * @returns The resolved value, typed.
 */
export function* call<T>(promise: Promise<T>): Generator<Promise<T>, T, T> {
  return yield promise;
}
