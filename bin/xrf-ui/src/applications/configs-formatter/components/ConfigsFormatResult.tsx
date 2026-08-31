import { GridColDef } from "@mui/x-data-grid";
import { ReactElement, useMemo } from "react";

import { LtxProjectFormatResult } from "@/core/bindings/types/xrf-ltx";
import { CommandResult, ICommandResultStat, TCommandResultTone } from "@/core/ui/command-result/CommandResult";
import { CommandResultFindings } from "@/core/ui/command-result/CommandResultFindings";
import { formatDuration } from "@/lib/format/duration";

interface IConfigsFormatResultProps {
  isCheck: boolean;
  result: LtxProjectFormatResult;
}

export function ConfigsFormatResult({ isCheck, result }: IConfigsFormatResultProps): ReactElement {
  const columns: Array<GridColDef> = useMemo(
    () => [{ field: "file", headerName: "File", flex: 1, minWidth: 320, cellClassName: "monospace" }],
    []
  );

  const rows: Array<{ file: string }> = useMemo(() => result.toFormat.map((file) => ({ file })), [result.toFormat]);

  const stats: Array<ICommandResultStat> = useMemo(
    () => [
      { label: "files", value: result.totalFiles },
      { label: "valid", value: result.validFiles, tone: "success" },
      {
        label: isCheck ? "need formatting" : "formatted",
        value: result.invalidFiles,
        tone: result.invalidFiles ? (isCheck ? "error" : "warning") : "success",
      },
      { label: "elapsed", value: formatDuration(result.duration) },
      { label: "opening", value: formatDuration(result.startupDuration) },
    ],
    [isCheck, result]
  );

  // In check mode a badly formatted file is a failure; in write mode the same number is work done.
  const tone: TCommandResultTone = result.invalidFiles ? (isCheck ? "error" : "warning") : "success";

  return (
    <CommandResult
      headline={
        result.invalidFiles
          ? isCheck
            ? `${result.invalidFiles} file(s) are not correctly formatted`
            : `Formatted ${result.invalidFiles} file(s)`
          : "All files are correctly formatted"
      }
      tone={tone}
      stats={stats}
    >
      <CommandResultFindings<{ file: string }>
        rows={rows}
        columns={columns}
        getRowId={(row) => row.file}
        getSearchText={(row) => row.file}
        emptyLabel={"Nothing to format."}
        searchPlaceholder={"Filter by file"}
      />
    </CommandResult>
  );
}
