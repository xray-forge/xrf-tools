import { useInjection } from "@wirestate/react";
import { ABSENT_VALUE, formatBytes, formatDuration } from "@xrf/format";
import { IIpcCommandMetrics, IIpcMetricsSnapshot } from "@xrf/ipc";
import { IpcMetricsService } from "@xrf/ipc/session";
import { Nullable } from "@xrf/types";
import { CheckboxFormRow } from "@xrf/ui/form/checkbox-row";
import { ChoiceFormRow, IChoiceFormRowOption } from "@xrf/ui/form/choice-row";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { Button } from "@xrf/ui/material/button";
import { Divider } from "@xrf/ui/material/divider";
import { Stack } from "@xrf/ui/material/stack";
import { Typography } from "@xrf/ui/material/typography";
import { StatFigure } from "@xrf/ui/stats";
import { ReactElement, useState } from "react";

import { usePolledValue } from "@/lib/react/use-polled-value";

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

  const [sort, setSort] = useState<EIpcSort>(EIpcSort.DURATION);

  const snapshot: IIpcMetricsSnapshot =
    usePolledValue(() => ipcMetricsService.read(), METRICS_POLL_INTERVAL) ?? ipcMetricsService.read();

  const listed: Array<IIpcCommandMetrics> = sortIpcCommands(snapshot.commands, sort).slice(0, COMMAND_LIMIT);

  return (
    <div className={"flex flex-col gap-6"}>
      <DetailSection
        title={"Backend calls"}
        description={"What this window has asked of the backend since it loaded."}
        fact={formatDuration(snapshot.elapsed)}
      >
        <div className={"mt-2 flex flex-wrap gap-4"}>
          <StatFigure label={"Calls"} value={String(snapshot.calls)} />
          <StatFigure label={"Failed"} value={String(snapshot.failures)} />
          <StatFigure label={"Received"} value={formatBytes(snapshot.received)} />
          <StatFigure label={"Sent"} value={snapshot.sent ? formatBytes(snapshot.sent) : "not weighed"} />
          <StatFigure label={"Time in flight"} value={formatDuration(snapshot.duration)} />
          <StatFigure label={"Peak at once"} value={String(snapshot.peakInFlight)} />
        </div>
      </DetailSection>

      <CheckboxFormRow
        label={"Weigh payloads"}
        description={
          "Measure what each JSON command carries, which means serializing it a second time to do so. Byte commands " +
          "are measured either way, because their size is already in hand."
        }
        isChecked={ipcMetricsService.isProfilingEnabled}
        onChange={ipcMetricsService.setProfilingEnabled}
      />

      <DetailSection
        title={"Commands"}
        description={"Every command called at least once, ordered by what it cost."}
        fact={snapshot.commands.length === 1 ? "1 command" : `${snapshot.commands.length} commands`}
      >
        <div className={"mt-2"}>
          <ChoiceFormRow label={"Order by"} options={SORT_OPTIONS} value={sort} onChange={setSort} />
        </div>

        {listed.length ? (
          <Stack className={"mt-2"} divider={<Divider flexItem />}>
            {listed.map((it: IIpcCommandMetrics) => {
              const weighed: Nullable<string> = describeWeighedCalls(it);

              return (
                <div key={it.command} className={"flex items-center gap-4 py-1.5"}>
                  <div className={"min-w-0 grow"}>
                    <Typography className={"monospace wrap-anywhere"}>{it.command}</Typography>

                    {it.failures ? (
                      <Typography className={"block text-warning"} variant={"caption"}>
                        {`${it.failures} failed, ${formatDuration(it.failureDuration)}`}
                      </Typography>
                    ) : null}
                  </div>

                  <Typography className={"min-w-19 shrink-0 text-right"} variant={"body2"}>
                    {it.calls}
                  </Typography>

                  <Typography className={"min-w-19 shrink-0 text-right"} variant={"body2"}>
                    {it.calls ? formatCallDuration(it.duration / it.calls) : ABSENT_VALUE}
                  </Typography>

                  <Typography className={"min-w-19 shrink-0 text-right"} variant={"body2"}>
                    {formatDuration(it.duration)}
                  </Typography>

                  <div className={"min-w-19 shrink-0 text-right"}>
                    <Typography variant={"body2"}>{it.weighed ? formatBytes(it.received) : ABSENT_VALUE}</Typography>

                    {weighed ? (
                      <Typography className={"block text-text-secondary"} variant={"caption"}>
                        {weighed}
                      </Typography>
                    ) : null}
                  </div>
                </div>
              );
            })}
          </Stack>
        ) : (
          <Typography className={"mt-2 block text-text-secondary"} variant={"caption"}>
            Nothing called yet.
          </Typography>
        )}

        <Divider className={"mt-4"} />

        <div className={"mt-4 flex"}>
          <Button color={"error"} size={"small"} variant={"outlined"} onClick={ipcMetricsService.reset}>
            Reset
          </Button>
        </div>
      </DetailSection>
    </div>
  );
}
