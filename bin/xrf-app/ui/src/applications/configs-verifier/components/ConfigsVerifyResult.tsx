import { formatCount, formatDuration } from "@xrf/format";
import { LtxFinding, LtxProjectVerifyResult } from "@xrf/ipc/types/xrf-ltx";
import { CommandResult, ICommandResultStat } from "@xrf/ui/command-result/CommandResult";
import { CommandResultFindings } from "@xrf/ui/command-result/CommandResultFindings";
import { GridColDef } from "@xrf/ui/data-grid";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { identifierColumn } from "@xrf/ui/table/columns";
import { ReactElement, useMemo } from "react";

import { toFindingMessage } from "@/core/ltx/lib/findings";

/**
 * One finding as a row, keyed by where it came in the run: two findings can say the same thing about the same place.
 */
interface IConfigsVerifyFindingRow extends LtxFinding {
  index: number;
}

interface IConfigsVerifyResultProps extends BaseComponentProps {
  result: LtxProjectVerifyResult;
}

export function ConfigsVerifyResult({
  "data-testid": dataTestId = "configs-verify-result",
  id,
  className,
  result,
}: IConfigsVerifyResultProps): ReactElement {
  const columns: Array<GridColDef> = useMemo(
    () => [
      identifierColumn("kind", "Kind", 110),
      identifierColumn("section", "Section", 180),
      identifierColumn("field", "Field", 150),
      {
        field: "message",
        headerName: "Problem",
        flex: 1,
        minWidth: 220,
        valueGetter: (_, row: IConfigsVerifyFindingRow) => toFindingMessage(row),
      },
      {
        field: "file",
        headerName: "Location",
        width: 220,
        cellClassName: "monospace",
        valueGetter: (_, row: IConfigsVerifyFindingRow) => row.file ?? row.entry,
      },
    ],
    []
  );

  const findings: Array<IConfigsVerifyFindingRow> = useMemo(
    () => result.findings.map((finding: LtxFinding, index: number) => ({ ...finding, index })),
    [result]
  );

  const stats: Array<ICommandResultStat> = useMemo(
    () => [
      { label: "files", value: result.totalFiles },
      { label: "sections checked", value: result.checkedSections },
      { label: "fields checked", value: result.checkedFields },
      { label: "valid", value: result.validSections, tone: "success" },
      { label: "skipped", value: result.skippedSections },
      { label: "invalid", value: result.invalidSections, tone: result.invalidSections ? "error" : "success" },
      { label: "elapsed", value: formatDuration(result.duration) },
      { label: "opening", value: formatDuration(result.startupDuration) },
    ],
    [result]
  );

  return (
    <CommandResult
      data-testid={dataTestId}
      id={id}
      className={className}
      headline={findings.length ? `${formatCount(findings.length)} problem(s) found` : "All sections passed validation"}
      tone={findings.length ? "error" : "success"}
      stats={stats}
    >
      <CommandResultFindings<IConfigsVerifyFindingRow>
        rows={findings}
        columns={columns}
        getRowId={(row) => row.index}
        getSearchText={(row) =>
          `${row.kind} ${row.section ?? ""} ${row.field ?? ""} ${row.message} ${row.file ?? ""} ${row.entry}`
        }
        emptyLabel={"Nothing to report."}
        searchPlaceholder={"Filter by section, field or file"}
      />
    </CommandResult>
  );
}
