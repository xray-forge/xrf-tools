import { Box, Button, Divider, Stack, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useEffect, useState } from "react";

import { IIpcCommandMetrics, IIpcMetricsSnapshot } from "@/core/ipc/metrics";
import { IpcMetricsService } from "@/core/ipc/services/metrics";
import { SettingsSection } from "@/core/settings/components/SettingsSection";
import { SettingsStat } from "@/core/settings/components/SettingsStat";
import { MONOSPACE } from "@/core/theme/tokens";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { ChoiceFormRow, IChoiceFormRowOption } from "@/core/ui/form/ChoiceFormRow";
import { formatDuration } from "@/lib/format/duration";
import { formatBytes } from "@/lib/memory/format";
import { Nullable } from "@/lib/types/general";

import {
  describeWeighedCalls,
  EIpcSort,
  formatCallDuration,
  IPC_SORT_LABELS,
  sortIpcCommands,
} from "./SettingsIpcSection.utils";

/** How often the panel asks the recorder where things stand. */
const METRICS_POLL_INTERVAL: number = 1000;
/** How many commands are listed before it stops, which no real session reaches and a runaway one would. */
const COMMAND_LIMIT: number = 128;

/** Room each measure column keeps, so the rows line up whatever they hold. */
const MEASURE_COLUMN_WIDTH: number = 76;

const SORT_OPTIONS: ReadonlyArray<IChoiceFormRowOption<EIpcSort>> = [
  EIpcSort.DURATION,
  EIpcSort.CALLS,
  EIpcSort.RECEIVED,
].map((value: EIpcSort) => ({ value, label: IPC_SORT_LABELS[value] }));

/**
 * What this window has asked of the backend, and what it cost.
 */
export function SettingsIpcSection(): ReactElement {
  const ipcMetricsService: IpcMetricsService = useInjection(IpcMetricsService);

  const [snapshot, setSnapshot] = useState<IIpcMetricsSnapshot>(() => ipcMetricsService.read());
  const [sort, setSort] = useState<EIpcSort>(EIpcSort.DURATION);

  const listed: Array<IIpcCommandMetrics> = sortIpcCommands(snapshot.commands, sort).slice(0, COMMAND_LIMIT);

  useEffect(() => {
    const timer: ReturnType<typeof setInterval> = setInterval(
      () => setSnapshot(ipcMetricsService.read()),
      METRICS_POLL_INTERVAL
    );

    return () => clearInterval(timer);
  }, [ipcMetricsService]);

  return (
    <Box sx={{ display: "flex", flexDirection: "column", gap: 3 }}>
      <SettingsSection
        title={"Backend calls"}
        description={"What this window has asked of the backend since it loaded."}
        fact={formatDuration(snapshot.elapsed)}
      >
        <Box sx={{ display: "flex", flexWrap: "wrap", gap: 2, marginTop: 1 }}>
          <SettingsStat label={"Calls"} value={String(snapshot.calls)} />
          <SettingsStat label={"Failed"} value={String(snapshot.failures)} />
          <SettingsStat label={"Received"} value={formatBytes(snapshot.received)} />
          <SettingsStat label={"Sent"} value={snapshot.sent ? formatBytes(snapshot.sent) : "not weighed"} />
          <SettingsStat label={"Time in flight"} value={formatDuration(snapshot.duration)} />
          <SettingsStat label={"Peak at once"} value={String(snapshot.peakInFlight)} />
        </Box>
      </SettingsSection>

      <CheckboxFormRow
        label={"Weigh payloads"}
        description={
          "Measure what each JSON command carries, which means serializing it a second time to do so. Byte commands " +
          "are measured either way, because their size is already in hand."
        }
        isChecked={ipcMetricsService.isProfilingEnabled}
        onChange={ipcMetricsService.setProfilingEnabled}
      />

      <SettingsSection
        title={"Commands"}
        description={"Every command called at least once, ordered by what it cost."}
        fact={snapshot.commands.length === 1 ? "1 command" : `${snapshot.commands.length} commands`}
      >
        <Box sx={{ marginTop: 1 }}>
          <ChoiceFormRow label={"Order by"} options={SORT_OPTIONS} value={sort} onChange={setSort} />
        </Box>

        {listed.length ? (
          <Stack divider={<Divider flexItem />} sx={{ marginTop: 1 }}>
            {listed.map((it: IIpcCommandMetrics) => {
              const weighed: Nullable<string> = describeWeighedCalls(it);

              return (
                <Box key={it.command} sx={{ alignItems: "center", display: "flex", gap: 2, paddingY: 0.75 }}>
                  <Box sx={{ flexGrow: 1, minWidth: 0 }}>
                    <Typography sx={{ ...MONOSPACE, overflowWrap: "anywhere" }}>{it.command}</Typography>

                    {it.failures ? (
                      <Typography variant={"caption"} sx={{ color: "warning.main", display: "block" }}>
                        {`${it.failures} failed, ${formatDuration(it.failureDuration)}`}
                      </Typography>
                    ) : null}
                  </Box>

                  <Typography
                    variant={"body2"}
                    sx={{ flexShrink: 0, minWidth: MEASURE_COLUMN_WIDTH, textAlign: "right" }}
                  >
                    {it.calls}
                  </Typography>

                  <Typography
                    variant={"body2"}
                    sx={{ flexShrink: 0, minWidth: MEASURE_COLUMN_WIDTH, textAlign: "right" }}
                  >
                    {it.calls ? formatCallDuration(it.duration / it.calls) : "—"}
                  </Typography>

                  <Typography
                    variant={"body2"}
                    sx={{ flexShrink: 0, minWidth: MEASURE_COLUMN_WIDTH, textAlign: "right" }}
                  >
                    {formatDuration(it.duration)}
                  </Typography>

                  <Box sx={{ flexShrink: 0, minWidth: MEASURE_COLUMN_WIDTH, textAlign: "right" }}>
                    <Typography variant={"body2"}>{it.weighed ? formatBytes(it.received) : "—"}</Typography>

                    {weighed ? (
                      <Typography variant={"caption"} sx={{ color: "text.secondary", display: "block" }}>
                        {weighed}
                      </Typography>
                    ) : null}
                  </Box>
                </Box>
              );
            })}
          </Stack>
        ) : (
          <Typography variant={"caption"} sx={{ color: "text.secondary", display: "block", marginTop: 1 }}>
            Nothing called yet.
          </Typography>
        )}

        <Divider sx={{ marginTop: 2 }} />

        <Box sx={{ display: "flex", marginTop: 2 }}>
          <Button color={"error"} size={"small"} variant={"outlined"} onClick={ipcMetricsService.reset}>
            Reset
          </Button>
        </Box>
      </SettingsSection>
    </Box>
  );
}
