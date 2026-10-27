import { formatBytes, formatDuration, formatInstant } from "@xrf/format";
import { systemCommands } from "@xrf/ipc/commands/system";
import { MemoryUsage, RuntimeSnapshot } from "@xrf/ipc/types/xrf-app-system";
import { Logger } from "@xrf/logging";
import { useLogger } from "@xrf/logging/react";
import { Nullable } from "@xrf/types";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { Typography } from "@xrf/ui/material/typography";
import { StatFigure } from "@xrf/ui/stats";
import { ReactElement } from "react";

import { useMemoryUsage } from "@/core/diagnostics/lib";
import { usePolledValue } from "@/lib/react/use-polled-value";

import { describeBackendMemory, describeWebviewMemory } from "./SettingsRuntimeSection.utils";

/**
 * When this session started, and what it is costing the machine right now.
 */
export function SettingsRuntimeSection(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const snapshot: Nullable<RuntimeSnapshot> = usePolledValue(
    () =>
      systemCommands.getRuntimeSnapshot().catch((error: unknown) => {
        log.error("Failed to read runtime snapshot:", error);

        return null;
      }),
    1_000
  );
  const memory: Nullable<MemoryUsage> = useMemoryUsage((usage: MemoryUsage): MemoryUsage => usage);

  return (
    <DetailSection
      data-testid={"settings-runtime-section"}
      title={"Runtime"}
      description={"When this session started, and what it is costing the machine right now."}
      fact={snapshot ? formatDuration(snapshot.uptime) : null}
    >
      {snapshot ? (
        <div className={"mt-2 flex flex-wrap gap-4"}>
          <StatFigure label={"Started"} value={formatInstant(snapshot.startedAt)} />
          <StatFigure label={"Uptime"} value={formatDuration(snapshot.uptime)} />
          <StatFigure label={"Backend"} {...describeBackendMemory(snapshot, memory)} />
          <StatFigure label={"Webview"} {...describeWebviewMemory(snapshot, memory)} />
          <StatFigure
            label={"Machine"}
            value={formatBytes(snapshot.machine.usedMemory)}
            hint={`${formatBytes(snapshot.machine.availableMemory)} free`}
          />
        </div>
      ) : (
        <Typography variant={"caption"}>Reading runtime details...</Typography>
      )}
    </DetailSection>
  );
}
