import { Injectable } from "@wirestate/core";
import { systemCommands } from "@xrf/ipc/commands/system";
import { MemoryUsage, RuntimeSnapshot } from "@xrf/ipc/types/xrf-app-system";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { PolledCommand } from "@/core/operations/lib/polled-command";

/** How often memory is read: slow enough to cost nothing, fast enough to watch a level stream in. */
const MEMORY_POLL_INTERVAL: number = 2_000;

/** How often the runtime snapshot is read, so its uptime ticks by the second. */
const RUNTIME_POLL_INTERVAL: number = 1_000;

/**
 * What the running application costs the machine right now, read while a view shows it: the status bar's memory and
 * the About section's runtime. A failed read is logged and clears the reading, as a platform that cannot say does.
 */
@Injectable()
export class RuntimeUsageService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** What the backend and the webview's processes hold, and the video memory; null where the platform cannot say. */
  public readonly memory: PolledCommand<MemoryUsage> = new PolledCommand(this.log, {
    read: () =>
      systemCommands.getMemoryUsage().catch((error: unknown): Nullable<MemoryUsage> => {
        this.log.error("Failed to read memory usage:", error);

        return null;
      }),
    interval: MEMORY_POLL_INTERVAL,
  });

  /** When the session started, its uptime, and the machine's memory. */
  public readonly snapshot: PolledCommand<RuntimeSnapshot> = new PolledCommand(this.log, {
    read: () =>
      systemCommands.getRuntimeSnapshot().catch((error: unknown): Nullable<RuntimeSnapshot> => {
        this.log.error("Failed to read runtime snapshot:", error);

        return null;
      }),
    interval: RUNTIME_POLL_INTERVAL,
  });
}
