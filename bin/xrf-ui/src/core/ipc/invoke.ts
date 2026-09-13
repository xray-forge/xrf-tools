import { InvokeArgs, InvokeOptions, invoke as invokeTauri } from "@tauri-apps/api/core";

import { IPC_METRICS, IpcCallMeasurement, weighIpcPayload } from "@/core/ipc/metrics";
import { Nullable } from "@/lib/types/general";

/**
 * Calls a command, counting what it cost.
 *
 * @param command - Fully qualified command name, such as `plugin:configs|read_document`.
 * @param rest - Arguments for the command, and Tauri's own invoke options.
 * @returns Whatever the command answered.
 */
export function invoke<T>(command: string, ...rest: [args?: InvokeArgs, options?: InvokeOptions]): Promise<T> {
  const call: IpcCallMeasurement = IPC_METRICS.measure(command);
  const response: Promise<T> = invokeTauri<T>(command, ...rest);

  void response.then(
    (value: T): void => {
      const received: Nullable<number> = call.isWeighing ? weighIpcPayload(value) : null;
      const sent: Nullable<number> = call.isWeighing ? weighIpcPayload(rest[0]) : null;

      call.recordAnswer(received, sent);
    },
    (): void => call.recordFailure()
  );

  return response;
}
