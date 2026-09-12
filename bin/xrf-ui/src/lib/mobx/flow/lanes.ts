import { Logger } from "@/lib/logging";
import { swallowCancellation } from "@/lib/mobx/flow/cancellation";
import { TCancellablePromise } from "@/lib/mobx/flow/types";
import { Nullable } from "@/lib/types/general";

/**
 * The run each flow decorated method currently has in flight, per instance.
 *
 * Held outside the instance so a service does not carry a field per lane whose only job is to be cancelled, which is
 * the hand-kept request counter this replaces.
 */
const RUNNING: WeakMap<object, Map<PropertyKey, TCancellablePromise<any>>> = new WeakMap();

/**
 * Replaces the running flow on a lane.
 *
 * @param instance - Instance owning the lane.
 * @param slot - Lane shared by the participating methods.
 * @param start - Starts work only after the previous flow is cancelled.
 * @returns The replacement flow's result, with cancellation settled quietly.
 */
export function runLatestFlow<T>(
  instance: object,
  slot: PropertyKey,
  start: () => TCancellablePromise<T>
): Promise<T | void> {
  Logger.info("Run latest flow:", slot);

  cancelLane(instance, slot);

  return trackFlow(instance, slot, start());
}

/**
 * Joins an occupied lane or starts work when it is free.
 *
 * @param instance - Instance owning the lane.
 * @param slot - Lane shared by the participating methods.
 * @param start - Starts work only when the lane is free.
 * @returns The running or newly started flow's result, with cancellation settled quietly.
 */
export function runExclusiveFlow<T>(
  instance: object,
  slot: PropertyKey,
  start: () => TCancellablePromise<T>
): Promise<T | void> {
  Logger.info("Run exclusive flow:", slot);

  const running: Nullable<TCancellablePromise<T>> = RUNNING.get(instance)?.get(slot) ?? null;

  if (running) {
    return running.catch(swallowCancellation);
  }

  return trackFlow(instance, slot, start());
}

/**
 * Cancels whatever a flow decorated method left running on one instance.
 *
 * What `clear`, `close` and deactivation call: the in-flight run is abandoned where it stands rather than allowed to
 * finish and write into state nobody is looking at any more.
 *
 * @param instance - Instance owning the run.
 * @param lane - Member the lane is named after: the field the run publishes to, or the method that owns it.
 */
export function cancelFlow<T extends object>(instance: T, lane: keyof T): void {
  Logger.info("Cancel flow:", lane);
  cancelLane(instance, lane as PropertyKey);
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

  for (const [lane, running] of slots.entries()) {
    Logger.info("Cancel flow:", lane);
    running.cancel();
  }

  slots.clear();
}

/**
 * Tracks a lane until its flow settles, preserving its result for the caller.
 *
 * @param instance - Instance owning the flow.
 * @param lane - Lane shared by the participating methods.
 * @param promise - Flow to track until completion or cancellation.
 * @returns The flow's result, with cancellation settled quietly.
 */
function trackFlow<T>(instance: object, lane: PropertyKey, promise: TCancellablePromise<T>): Promise<T | void> {
  const slots: Map<PropertyKey, TCancellablePromise<any>> = RUNNING.get(instance) ?? new Map();

  slots.set(lane, promise);
  RUNNING.set(instance, slots);

  function forget(): void {
    if (RUNNING.get(instance)?.get(lane) === promise) {
      RUNNING.get(instance)?.delete(lane);
    }
  }

  return promise.then(
    (result: T) => {
      forget();

      return result;
    },
    (error: unknown) => {
      forget();

      return swallowCancellation(error);
    }
  );
}

/**
 * Cancels one lane without the caller having to name it as a member.
 *
 * @param instance - Instance owning the run.
 * @param lane - Lane to abandon.
 */
function cancelLane(instance: object, lane: PropertyKey): void {
  const running: Nullable<TCancellablePromise<any>> = RUNNING.get(instance)?.get(lane) ?? null;

  running?.cancel();
  RUNNING.get(instance)?.delete(lane);
}
