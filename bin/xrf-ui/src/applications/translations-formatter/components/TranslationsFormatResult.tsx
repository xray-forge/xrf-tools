import { GridColDef } from "@mui/x-data-grid";
import { ReactElement, useMemo } from "react";

import { ProjectFormatResult } from "@/core/bindings/types/xrf-translation";
import { CommandResult, ICommandResultStat, TCommandResultTone } from "@/core/ui/command-result/CommandResult";
import { CommandResultFindings } from "@/core/ui/command-result/CommandResultFindings";
import { formatDuration } from "@/lib/format/duration";

interface ITranslationsFormatResultProps {
  isCheck: boolean;
  result: ProjectFormatResult;
}

export function TranslationsFormatResult({ isCheck, result }: ITranslationsFormatResultProps): ReactElement {
  const columns: Array<GridColDef> = useMemo(
    () => [{ field: "file", headerName: "Source", flex: 1, minWidth: 320, cellClassName: "monospace" }],
    []
  );

  const rows: Array<{ file: string }> = useMemo(() => result.toFormat.map((file) => ({ file })), [result.toFormat]);

  const stats: Array<ICommandResultStat> = useMemo(
    () => [
      { label: "sources", value: result.totalFiles },
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

  // In check mode an unformatted source is a failure; in write mode the same number is work done.
  const tone: TCommandResultTone = result.invalidFiles ? (isCheck ? "error" : "warning") : "success";

  return (
    <CommandResult
      headline={
        result.invalidFiles
          ? isCheck
            ? `${result.invalidFiles} source(s) are not correctly formatted`
            : `Formatted ${result.invalidFiles} source(s)`
          : "All translation sources are correctly formatted"
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
        searchPlaceholder={"Filter by source"}
      />
    </CommandResult>
  );
}
