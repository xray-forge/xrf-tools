import { invoke } from "@tauri-apps/api/core";

import { IPC_METRICS, IpcCallMeasurement, weighIpcPayload } from "@/core/ipc/metrics";
import { Nullable } from "@/lib/types/general";

/**
 * Call a command that answers with bytes rather than a typed value.
 *
 * A command returning `tauri::ipc::Response` cannot be Specta typed, so these have generated wrappers in
 * `core/bindings/` that route through here instead of through the Specta output. Tauri's custom protocol serves the
 * body as `application/octet-stream`, which the injected script hands over as an `ArrayBuffer` in one transfer, with no
 * base64 inflation and no json parse.
 *
 * When that protocol is unavailable the script silently falls back to `postMessage`, where the body arrives as
 * something else entirely, so the type is asserted rather than trusted: the failure mode otherwise is a plausible
 * looking result built from nonsense.
 *
 * @param command - Fully qualified command name, such as `plugin:visuals|read_geometry`.
 * @param args - Arguments for the command.
 * @returns The raw response bytes.
 */
export async function invokeRaw(command: string, args: Record<string, unknown>): Promise<ArrayBuffer> {
  const call: IpcCallMeasurement = IPC_METRICS.measure(command);
  const sent: Nullable<number> = call.isWeighing ? weighIpcPayload(args) : null;

  let response: unknown;

  try {
    response = await invoke<unknown>(command, args);
  } catch (error: unknown) {
    call.recordFailure();

    throw error;
  }

  if (response instanceof ArrayBuffer) {
    call.recordAnswer(response.byteLength, sent);

    return response;
  }

  // A typed array would still be usable, but only by accident, so it is converted explicitly.
  if (response instanceof Uint8Array) {
    call.recordAnswer(response.byteLength, sent);

    return response.buffer.slice(response.byteOffset, response.byteOffset + response.byteLength) as ArrayBuffer;
  }

  // The command answered, but not with what it promised, which is a failure of this call rather than of the backend.
  call.recordFailure();

  throw new Error(
    `Expected raw bytes from '${command}', got ${typeof response}. ` +
      "The tauri custom protocol is likely unavailable, so the raw response could not be transferred."
  );
}
