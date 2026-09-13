import { IPC_PROFILING_STORAGE_KEY } from "@/core/storage";
import { getLocalStorageValue, setLocalStorageValue } from "@/lib/local-storage";

/** Read once, here, rather than when a container provisions something. */
let isProfiling: boolean = getLocalStorageValue(IPC_PROFILING_STORAGE_KEY) === String(true);

/** @returns Whether payloads are being weighed. */
export function isIpcProfilingEnabled(): boolean {
  return isProfiling;
}

/**
 * @param isEnabled - Whether calls from here on weigh their payloads.
 */
export function setIpcProfilingEnabled(isEnabled: boolean): void {
  isProfiling = isEnabled;

  setLocalStorageValue(IPC_PROFILING_STORAGE_KEY, String(isEnabled));
}
