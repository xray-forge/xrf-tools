import { setIpcProfilingEnabled, subscribeToIpcProfiling } from "@xrf/ipc";

import { getLocalStorageValue, IPC_PROFILING_STORAGE_KEY, setLocalStorageValue } from "@/core/storage";

/**
 * Restores whether IPC payloads are weighed from the preference that keeps it, and keeps each change from here on.
 *
 * Called once as the page starts, before anything calls the backend: profiling a startup means the switch has to
 * survive the reload that reproduces it, and a call made during startup already knows whether it weighs itself.
 *
 * @returns Stops keeping changes, for a test.
 */
export function persistIpcProfiling(): () => void {
  setIpcProfilingEnabled(getLocalStorageValue(IPC_PROFILING_STORAGE_KEY) === String(true));

  return subscribeToIpcProfiling((isEnabled: boolean) =>
    setLocalStorageValue(IPC_PROFILING_STORAGE_KEY, String(isEnabled))
  );
}
