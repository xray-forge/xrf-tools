import { flow, isFlowCancellationError } from "@wirestate/mobx";

import { Nullable } from "@/lib/types/general";

/**
 * A flow's promise, which can be told the answer is no longer wanted.
 */
type TCancellablePromise<T> = Promise<T> & { cancel: () => void };

/** Generator shape a flow decorated method has. */
type TFlowGenerator = (...args: Array<any>) => Generator<any, any, any>;

/**
 * Return type of a flow decorated method.
 *
 * The sent-in type has to stay permissive so `yield* call(...)` can delegate: the outer generator cannot know what a
 * delegated one will be handed back, and a narrower declaration refuses the delegation rather than checking it.
 */
export type TFlow<R = void> = Generator<any, R, any>;

/**
 * The run each flow decorated method currently has in flight, per instance.
 *
 * Held outside the instance so a service does not carry a field per lane whose only job is to be cancelled, which is
 * the hand-kept request counter this replaces.
 */
const RUNNING: WeakMap<object, Map<PropertyKey, TCancellablePromise<any>>> = new WeakMap();

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

/**
 * Reports whether a rejection is a flow that was cancelled rather than a failure.
 *
 * @param error - Rejection to classify.
 * @returns Whether the flow was cancelled.
 */
export function isCancellation(error: unknown): boolean {
  return isFlowCancellationError(error as Error);
}

/**
 * Lets a cancellation settle quietly while a real failure still reaches the caller.
 *
 * @param error - Rejection the flow produced.
 */
function swallowCancellation(error: unknown): void {
  if (!isFlowCancellationError(error as Error)) {
    throw error;
  }
}

/**
 * Publishes a wrapped flow as a method bound to its instance on first read.
 *
 * Bound the way `action.bound` binds, because these are handed to React as callbacks: a component passing
 * `onLoad={service.loadPatrols}` detaches the method from its instance, and an unbound one would run with no `this`.
 *
 * @param key - Method name being replaced.
 * @param run - Wrapped implementation to bind.
 * @returns A property descriptor that binds on first access.
 */
function toBoundDescriptor(key: PropertyKey, run: (...args: Array<any>) => Promise<any>): TypedPropertyDescriptor<any> {
  return {
    configurable: true,
    get(this: object): (...args: Array<any>) => Promise<any> {
      const bound: (...args: Array<any>) => Promise<any> = run.bind(this);

      Object.defineProperty(this, key, { configurable: true, value: bound, writable: false });

      return bound;
    },
  };
}

/**
 * Runs a generator method as a flow, cancelling whatever the previous call left running.
 *
 * Cancelling resumes the generator with a return completion, so the lines after the `yield` in flight never execute -
 * a superseded run cannot publish, rather than publishing and being compared away afterwards. `finally` blocks still
 * run, which is where a lane releases anything it had already taken.
 *
 * @param lane - Name shared by every method feeding one lane, so they supersede each other. Defaults to the method's
 *   own name, which is right when a lane has a single entry point. `load` and `restore` both fill the open visual, so
 *   they name the lane instead: a restore that lands after a load must not publish over it.
 * @returns The method decorator that wraps the generator.
 */
export function LatestFlow(lane?: PropertyKey): MethodDecorator {
  return function decorateLatestFlow(
    _target: object,
    key: PropertyKey,
    descriptor: TypedPropertyDescriptor<any>
  ): TypedPropertyDescriptor<any> {
    const slot: PropertyKey = lane ?? key;
    const runner: (...args: Array<any>) => TCancellablePromise<any> = flow(descriptor.value as TFlowGenerator);

    function runLatest(this: object, ...args: Array<any>): Promise<any> {
      cancelFlow(this, slot);

      const promise: TCancellablePromise<any> = runner.apply(this, args);
      const slots: Map<PropertyKey, TCancellablePromise<any>> = RUNNING.get(this) ?? new Map();

      slots.set(slot, promise);
      RUNNING.set(this, slots);

      return promise.catch(swallowCancellation);
    }

    return toBoundDescriptor(key, runLatest);
  };
}

/**
 * Runs a generator method as a flow, ignoring the call entirely while one is already running.
 *
 * The other half of {@link LatestFlow}, and not interchangeable with it. Supersede is right when a newer request
 * replaces an older one - a different file, a different motion. Ignore is right when every call asks for the same
 * thing, which is what a view asking for its chunk on mount does: superseding there would cancel a run that had
 * already published a loading state, leaving the lane loading with nothing left to finish it.
 *
 * @param lane - Name shared by every method feeding one lane. Defaults to the method's own name.
 * @returns The method decorator that wraps the generator.
 */
export function ExclusiveFlow(lane?: PropertyKey): MethodDecorator {
  return function decorateExclusiveFlow(
    _target: object,
    key: PropertyKey,
    descriptor: TypedPropertyDescriptor<any>
  ): TypedPropertyDescriptor<any> {
    const slot: PropertyKey = lane ?? key;
    const runner: (...args: Array<any>) => TCancellablePromise<any> = flow(descriptor.value as TFlowGenerator);

    function runExclusive(this: object, ...args: Array<any>): Promise<any> {
      const running: Nullable<TCancellablePromise<any>> = RUNNING.get(this)?.get(slot) ?? null;

      if (running) {
        return running.catch(swallowCancellation);
      }

      const promise: TCancellablePromise<any> = runner.apply(this, args);
      const slots: Map<PropertyKey, TCancellablePromise<any>> = RUNNING.get(this) ?? new Map();

      slots.set(slot, promise);
      RUNNING.set(this, slots);

      // Cleared when it settles, so the lane is askable again. `LatestFlow` has no equivalent because its next call
      // replaces the entry outright.
      const forget: () => void = () => {
        if (RUNNING.get(this)?.get(slot) === promise) {
          RUNNING.get(this)?.delete(slot);
        }
      };

      return promise.then(forget, (error: unknown) => {
        forget();

        return swallowCancellation(error);
      });
    }

    return toBoundDescriptor(key, runExclusive);
  };
}

/**
 * Cancels whatever a flow decorated method left running on one instance.
 *
 * What `clear`, `close` and deactivation call: the in-flight run is abandoned where it stands rather than allowed to
 * finish and write into state nobody is looking at any more.
 *
 * @param instance - Instance owning the run.
 * @param key - Name of the flow decorated method.
 */
export function cancelFlow(instance: object, key: PropertyKey): void {
  const running = RUNNING.get(instance)?.get(key) ?? null;

  running?.cancel();
  RUNNING.get(instance)?.delete(key);
}

/**
 * Cancels every flow left running on one instance.
 *
 * @param instance - Instance whose runs should be abandoned.
 */
export function cancelFlows(instance: object): void {
  const slots = RUNNING.get(instance);

  if (!slots) {
    return;
  }

  for (const running of slots.values()) {
    running.cancel();
  }

  slots.clear();
}
