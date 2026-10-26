import { flow } from "@wirestate/mobx";

import { runExclusiveFlow, runLatestFlow } from "@/lib/mobx/flow/lanes";
import { TCancellablePromise } from "@/lib/mobx/flow/types";

/**
 * A flow decorator, with the decorated class in an inference position.
 */
type TFlowDecorator<T> = (
  target: T,
  key: PropertyKey,
  descriptor: TypedPropertyDescriptor<any>
) => TypedPropertyDescriptor<any>;

/** Generator shape a flow decorated method has. */
type TFlowGenerator = (...args: Array<any>) => Generator<any, any, any>;

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
export function LatestFlow<T = object>(lane?: keyof T): TFlowDecorator<T> {
  return function decorateLatestFlow(
    _target: T,
    key: PropertyKey,
    descriptor: TypedPropertyDescriptor<any>
  ): TypedPropertyDescriptor<any> {
    const slot: PropertyKey = (lane as PropertyKey) ?? key;
    const runner: (...args: Array<any>) => TCancellablePromise<any> = flow(descriptor.value as TFlowGenerator);

    function runLatest(this: object, ...args: Array<any>): Promise<any> {
      return runLatestFlow(this, slot, () => runner.apply(this, args));
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
export function ExclusiveFlow<T = object>(lane?: keyof T): TFlowDecorator<T> {
  return function decorateExclusiveFlow(
    _target: T,
    key: PropertyKey,
    descriptor: TypedPropertyDescriptor<any>
  ): TypedPropertyDescriptor<any> {
    const slot: PropertyKey = (lane as PropertyKey) ?? key;
    const runner: (...args: Array<any>) => TCancellablePromise<any> = flow(descriptor.value as TFlowGenerator);

    function runExclusive(this: object, ...args: Array<any>): Promise<any> {
      return runExclusiveFlow(this, slot, () => runner.apply(this, args));
    }

    return toBoundDescriptor(key, runExclusive);
  };
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
