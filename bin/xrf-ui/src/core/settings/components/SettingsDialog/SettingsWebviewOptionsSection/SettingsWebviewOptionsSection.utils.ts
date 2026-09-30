import { WebviewOptions, WebviewOptionsStatus } from "@/core/ipc/types/xrf-app";

/**
 * @param status - The options the webview runs with, and those chosen for the next start.
 * @returns Whether a choice waits for a restart: the browser takes its options only as it starts.
 */
export function isRestartPending(status: WebviewOptionsStatus): boolean {
  const { running, chosen }: WebviewOptionsStatus = status;

  return (Object.keys(chosen) as Array<keyof WebviewOptions>).some((key) => chosen[key] !== running[key]);
}
