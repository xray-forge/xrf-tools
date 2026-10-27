import { formatDuration } from "@xrf/format";
import { GamedataCheckSummary, GamedataVerifySummary } from "@xrf/ipc/types/xrf-app-gamedata";
import { EJobOutcome } from "@xrf/ipc/types/xrf-job";
import { EStatus, Status } from "@xrf/ipc/types/xrf-report";
import { GridColDef, GridRenderCellParams } from "@xrf/ui/data-grid";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { cn } from "@xrf/ui/theme/class-name";
import { ReactElement, useMemo } from "react";

import { selectFailedChecks } from "@/applications/gamedata-verifier/lib/describe-gamedata-verify-outcome";
import { CommandResult, ICommandResultStat, TCommandResultTone } from "@/core/ui/command-result/CommandResult";
import { CommandResultFindings } from "@/core/ui/command-result/CommandResultFindings";
import { identifierColumn } from "@/core/ui/table/columns";

/**
 * How each verdict reads in the tone system.
 */
const STATUS_TONES: Record<Status, TCommandResultTone> = {
  [EStatus.PASSED]: "success",
  [EStatus.FAILED]: "error",
  [EStatus.ERROR]: "error",
  [EStatus.INCOMPLETE]: "warning",
  [EStatus.SKIPPED]: "info",
};

interface IGamedataVerifyResultProps extends BaseComponentProps {
  result: GamedataVerifySummary;
}

export function GamedataVerifyResult({
  "data-testid": dataTestId = "gamedata-verify-result",
  id,
  className,
  result,
}: IGamedataVerifyResultProps): ReactElement {
  const columns: Array<GridColDef> = useMemo(
    () => [
      identifierColumn("check", "Check", 160),
      {
        field: "status",
        headerName: "Verdict",
        width: 120,
        renderCell: ({ row }: GridRenderCellParams<GamedataCheckSummary>) => {
          const tone: TCommandResultTone = STATUS_TONES[row.status];

          return (
            <span
              className={cn(
                tone === "success" ? "text-success" : null,
                tone === "warning" ? "text-warning" : null,
                tone === "error" ? "text-error" : null,
                tone === "info" ? "text-info" : null
              )}
            >
              {row.status}
            </span>
          );
        },
      },
      { field: "findings", headerName: "Findings", width: 110 },
      { field: "summary", headerName: "Summary", flex: 1, minWidth: 260 },
      { field: "duration", headerName: "Elapsed", width: 110 },
    ],
    []
  );

  const rows = useMemo(
    () =>
      result.checks.map((check: GamedataCheckSummary, index: number) => ({
        id: index,
        check: check.check,
        status: check.status,
        findings: check.findings,
        summary: check.summary,
        duration: check.duration === null ? "" : formatDuration(check.duration),
      })),
    [result]
  );

  const failed: Array<GamedataCheckSummary> = useMemo(() => selectFailedChecks(result), [result]);

  const stats: Array<ICommandResultStat> = useMemo(
    () => [
      { label: "checks", value: result.checks.length },
      { label: "failed", value: failed.length, tone: failed.length ? "error" : "success" },
      {
        label: "findings",
        value: result.checks.reduce((total: number, check: GamedataCheckSummary) => total + check.findings, 0),
      },
      { label: "elapsed", value: formatDuration(result.duration) },
    ],
    [failed, result]
  );

  const isCancelled: boolean = result.outcome === EJobOutcome.CANCELLED;

  return (
    <CommandResult
      data-testid={dataTestId}
      id={id}
      className={className}
      headline={
        isCancelled
          ? "Verification stopped — some checks are incomplete or were not run"
          : failed.length
            ? `Gamedata ${result.status}: ${failed.length} of ${result.checks.length} check(s)`
            : `Gamedata passed ${result.checks.length} check(s)`
      }
      // A stopped run is never a verdict, whatever the checks that did run said.
      tone={isCancelled ? "info" : STATUS_TONES[result.status]}
      stats={stats}
    >
      <CommandResultFindings
        columns={columns}
        rows={rows}
        getRowId={(row) => row.id}
        getSearchText={(row) => `${row.check} ${row.status} ${row.summary}`}
        emptyLabel={"No checks ran"}
        searchPlaceholder={"Filter checks"}
      />
    </CommandResult>
  );
}
