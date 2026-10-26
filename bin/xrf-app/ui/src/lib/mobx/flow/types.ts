/**
 * A flow's promise, which can be told the answer is no longer wanted.
 */
export type TCancellablePromise<T> = Promise<T> & { cancel: () => void };

/**
 * Return type of a flow decorated method.
 *
 * The sent-in type has to stay permissive so `yield* call(...)` can delegate: the outer generator cannot know what a
 * delegated one will be handed back, and a narrower declaration refuses the delegation rather than checking it.
 */
export type TFlow<R = void> = Generator<any, R, any>;
