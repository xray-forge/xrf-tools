import { GridColDef } from "@mui/x-data-grid";
import { ReactElement, useMemo } from "react";

import { LtxProjectVerifyResult } from "@/core/bindings/types/xrf-ltx";
import { TLtxSchemeError, toLtxSchemeErrors } from "@/core/ltx";
import { CommandResult, ICommandResultStat } from "@/core/ui/command-result/CommandResult";
import { CommandResultFindings } from "@/core/ui/command-result/CommandResultFindings";
import { formatDuration } from "@/lib/format/duration";

interface IConfigsVerifyResultProps {
  result: LtxProjectVerifyResult;
}

export function ConfigsVerifyResult({ result }: IConfigsVerifyResultProps): ReactElement {
  const columns: Array<GridColDef> = useMemo(
    () => [
      { field: "section", headerName: "Section", width: 180, cellClassName: "monospace" },
      { field: "field", headerName: "Field", width: 150, cellClassName: "monospace" },
      { field: "message", headerName: "Problem", flex: 1, minWidth: 220 },
      { field: "at", headerName: "Location", width: 220, cellClassName: "monospace" },
    ],
    []
  );

  const findings: Array<TLtxSchemeError> = useMemo(() => toLtxSchemeErrors(result.errors), [result]);

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
      headline={
        findings.length
          ? `${findings.length} problem(s) found in ${result.invalidSections} section(s)`
          : "All sections passed validation"
      }
      tone={findings.length ? "error" : "success"}
      stats={stats}
    >
      <CommandResultFindings<TLtxSchemeError>
        rows={findings}
        columns={columns}
        getRowId={(row) => `${row.at}:${row.section}:${row.field}`}
        getSearchText={(row) => `${row.section} ${row.field} ${row.message} ${row.at}`}
        emptyLabel={"Nothing to report."}
        searchPlaceholder={"Filter by section, field or file"}
      />
    </CommandResult>
  );
}
