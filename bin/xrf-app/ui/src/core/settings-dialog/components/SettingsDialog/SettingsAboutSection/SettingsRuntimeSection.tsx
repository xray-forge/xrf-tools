import { useInjection } from "@wirestate/react";
import { formatBytes, formatDuration, formatInstant } from "@xrf/format";
import { MemoryUsage, RuntimeSnapshot } from "@xrf/ipc/types/xrf-app-system";
import { Nullable } from "@xrf/types";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { StatFigure } from "@xrf/ui/stats";
import { Typography } from "@xrf/ui/typography";
import { ReactElement } from "react";

import { useMemoryUsage } from "@/core/diagnostics/lib/use-memory-usage";
import { RuntimeUsageService } from "@/core/diagnostics/services/runtime-usage.service";
import { usePolledCommand } from "@/core/operations/lib/use-polled-command";

import { describeBackendMemory, describeWebviewMemory } from "./SettingsRuntimeSection.utils";

/**
 * When this session started, and what it is costing the machine right now.
 */
export function SettingsRuntimeSection(): ReactElement {
  const runtimeUsageService: RuntimeUsageService = useInjection(RuntimeUsageService);

  const snapshot: Nullable<RuntimeSnapshot> = usePolledCommand(runtimeUsageService.snapshot);
  const memory: Nullable<MemoryUsage> = useMemoryUsage();

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
