import {
  EWebviewCollectionPace,
  WebviewCollectionPace,
  WebviewOptions,
  WebviewOptionsStatus,
} from "@/core/ipc/types/xrf-app";
import { IChoiceFormRowOption } from "@/core/ui/form";

/**
 * @param status - The options the webview runs with, and those chosen for the next start.
 * @returns Whether a choice waits for a restart: the browser takes its options only as it starts.
 */
export function isRestartPending(status: WebviewOptionsStatus): boolean {
  const { running, chosen }: WebviewOptionsStatus = status;

  return (Object.keys(chosen) as Array<keyof WebviewOptions>).some((key) => chosen[key] !== running[key]);
}

/** The collection paces offered, V8's own first; frequent is the backend's default. */
export const WEBVIEW_COLLECTION_PACE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<WebviewCollectionPace>> = [
  { label: "V8's own", value: EWebviewCollectionPace.DEFAULT },
  { label: "Earlier", value: EWebviewCollectionPace.EARLIER },
  { label: "Frequent", value: EWebviewCollectionPace.FREQUENT },
];
