import { Nullable } from "@xrf/types";

import { transformError } from "@/core/error/lib";
import { IBulkCall } from "@/core/ipc/bulk/bulk-call";
import { IBulkRequest } from "@/core/ipc/bulk/bulk-request";
import { requestBulk } from "@/core/ipc/bulk/request-bulk";
import { IPC_METRICS, IpcCallMeasurement, weighIpcPayload } from "@/core/ipc/metrics";

/**
 * Fetches a bulk route over the loopback transport, counting what it cost the way an IPC call is counted.
 *
 * The bytes never cross the window's thread, which is what Tauri's IPC answers every call on; a failed route rejects
 * with the message its command would have rejected with.
 *
 * @param call - A route and its arguments.
 * @returns The bytes it answered.
 */
export async function fetchBulk(call: IBulkCall): Promise<ArrayBuffer> {
  const measurement: IpcCallMeasurement = IPC_METRICS.measure(toBulkCommandName(call.route));
  const sent: Nullable<number> = measurement.isWeighing ? weighIpcPayload(call.args) : null;

  let bytes: ArrayBuffer;

  try {
    const request: IBulkRequest = await requestBulk(call);
    const response: Response = await fetch(request.url, {
      body: request.body,
      headers: request.headers,
      method: "POST",
    }).catch((error: unknown) => {
      throw new Error(`The transport did not answer '${call.route}': ${transformError(error).message}`);
    });

    if (!response.ok) {
      throw new Error(await readBulkFailure(response));
    }

    bytes = await response.arrayBuffer();
  } catch (error: unknown) {
    measurement.recordFailure();

    throw error;
  }

  measurement.recordAnswer(bytes.byteLength, sent);

  return bytes;
}

/**
 * @param route - `<plugin>/<route>`.
 * @returns The name the metrics count it under, which is the one its IPC command had.
 */
function toBulkCommandName(route: string): string {
  return route.replace("/", "|");
}

/**
 * @param response - A route's refusal.
 * @returns Its message: the JSON string the backend answers with, or the status where the body is not one.
 */
async function readBulkFailure(response: Response): Promise<string> {
  const text: string = await response.text().catch(() => "");

  try {
    const message: unknown = JSON.parse(text);

    if (typeof message === "string") {
      return message;
    }
  } catch {
    // Not the backend's shape, so the status is all there is to say.
  }

  return `The transport answered ${response.status} ${response.statusText}`.trim();
}
