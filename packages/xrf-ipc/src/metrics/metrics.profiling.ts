/** What is told when profiling is turned on or off. */
export type IpcProfilingListener = (isEnabled: boolean) => void;

/**
 * Whether payloads are weighed: off until the application restores the choice it keeps, before the first call it
 * should apply to.
 */
let isProfiling: boolean = false;

const listeners: Set<IpcProfilingListener> = new Set();

/** @returns Whether payloads are being weighed. */
export function isIpcProfilingEnabled(): boolean {
  return isProfiling;
}

/**
 * @param isEnabled - Whether calls from here on weigh their payloads; each listener is told of a change.
 */
export function setIpcProfilingEnabled(isEnabled: boolean): void {
  if (isProfiling === isEnabled) {
    return;
  }

  isProfiling = isEnabled;

  for (const listener of listeners) {
    listener(isEnabled);
  }
}

/**
 * Tells `listener` of every change to whether payloads are weighed, so the application can keep the choice.
 *
 * @param listener - Told the new state after each change.
 * @returns Stops telling it.
 */
export function subscribeToIpcProfiling(listener: IpcProfilingListener): () => void {
  listeners.add(listener);

  return () => {
    listeners.delete(listener);
  };
}
