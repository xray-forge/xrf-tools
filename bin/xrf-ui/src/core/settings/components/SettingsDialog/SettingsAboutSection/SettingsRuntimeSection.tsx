import { Typography } from "@mui/material";
import { ReactElement } from "react";

import { systemCommands } from "@/core/ipc/commands/system";
import { RuntimeSnapshot } from "@/core/ipc/types/xrf-app";
import { DetailSection } from "@/core/ui/layout/DetailSection";
import { StatFigure } from "@/core/ui/stats/StatFigure";
import { formatDuration } from "@/lib/format/duration";
import { formatInstant } from "@/lib/format/instant";
import { Logger, useLogger } from "@/lib/logging";
import { formatBytes } from "@/lib/memory/format";
import { usePolledValue } from "@/lib/react";
import { Nullable } from "@/lib/types/general";

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
          <StatFigure label={"Backend"} value={formatBytes(snapshot.process.residentMemory)} hint={"resident"} />
          <StatFigure
            label={"Webview"}
            value={formatBytes(snapshot.descendants.residentMemory)}
            hint={snapshot.descendants.processes === 1 ? "1 process" : `${snapshot.descendants.processes} processes`}
          />
          <StatFigure label={"Address space"} value={formatBytes(snapshot.process.virtualMemory)} hint={"reserved"} />
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
