import { Box, Typography } from "@mui/material";
import { ReactElement } from "react";

import { systemCommands } from "@/core/bindings/commands/system";
import { RuntimeSnapshot } from "@/core/bindings/types/xrf-app";
import { SettingsSection } from "@/core/settings/components/SettingsSection";
import { SettingsStat } from "@/core/settings/components/SettingsStat";
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
    <SettingsSection
      data-testid={"settings-runtime-section"}
      title={"Runtime"}
      description={"When this session started, and what it is costing the machine right now."}
      fact={snapshot ? formatDuration(snapshot.uptime) : null}
    >
      {snapshot ? (
        <Box sx={{ display: "flex", flexWrap: "wrap", gap: 2, marginTop: 1 }}>
          <SettingsStat label={"Started"} value={formatInstant(snapshot.startedAt)} />
          <SettingsStat label={"Uptime"} value={formatDuration(snapshot.uptime)} />
          <SettingsStat label={"Backend"} value={formatBytes(snapshot.process.residentMemory)} hint={"resident"} />
          <SettingsStat
            label={"Webview"}
            value={formatBytes(snapshot.descendants.residentMemory)}
            hint={snapshot.descendants.processes === 1 ? "1 process" : `${snapshot.descendants.processes} processes`}
          />
          <SettingsStat label={"Address space"} value={formatBytes(snapshot.process.virtualMemory)} hint={"reserved"} />
          <SettingsStat
            label={"Machine"}
            value={formatBytes(snapshot.machine.usedMemory)}
            hint={`${formatBytes(snapshot.machine.availableMemory)} free`}
          />
        </Box>
      ) : (
        <Typography variant={"caption"}>Reading runtime details...</Typography>
      )}
    </SettingsSection>
  );
}
