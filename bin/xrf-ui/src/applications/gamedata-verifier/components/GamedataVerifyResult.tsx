import { GridColDef } from "@mui/x-data-grid";
import { ReactElement, useMemo } from "react";

import { selectFailedChecks } from "@/applications/gamedata-verifier/lib/describe-gamedata-verify-outcome";
import { GamedataCheckSummary, GamedataVerifySummary } from "@/core/bindings/types/xrf-app";
import { CommandResult, ICommandResultStat, TCommandResultTone } from "@/core/ui/command-result/CommandResult";
import { CommandResultFindings } from "@/core/ui/command-result/CommandResultFindings";
import { formatDuration } from "@/lib/format/duration";

/**
 * How each verdict reads in the tone system.
 */
const STATUS_TONES: Record<string, TCommandResultTone> = {
  passed: "success",
  failed: "error",
  incomplete: "warning",
  skipped: "info",
};

interface IGamedataVerifyResultProps {
  result: GamedataVerifySummary;
}

export function GamedataVerifyResult({ result }: IGamedataVerifyResultProps): ReactElement {
  const columns: Array<GridColDef> = useMemo(
    () => [
      { field: "check", headerName: "Check", width: 160, cellClassName: "monospace" },
      { field: "status", headerName: "Verdict", width: 120 },
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

  return (
    <CommandResult
      headline={
        result.outcome === "cancelled"
          ? `Stopped after ${result.checks.length} check(s) — the rest were not run`
          : failed.length
            ? `Gamedata ${result.status}: ${failed.length} of ${result.checks.length} check(s)`
            : `Gamedata passed ${result.checks.length} check(s)`
      }
      // A stopped run is never a verdict, whatever the checks that did run said.
      tone={result.outcome === "cancelled" ? "info" : (STATUS_TONES[result.status] ?? "info")}
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
