import { getLocalStorageValue, setLocalStorageValue } from "@/lib/local-storage";

/**
 * Whether payloads are weighed, remembered between sessions.
 *
 * todo: Decide on separator for LS flags.
 */
export const IPC_PROFILING_STORAGE_KEY: string = "xrf-ipc-profiling";

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
