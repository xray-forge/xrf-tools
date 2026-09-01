import { GridColDef } from "@mui/x-data-grid";
import { ReactElement, useMemo } from "react";

import { TranslationVerifySummary } from "@/core/bindings/types/xrf-app";
import { TranslationVerifyLanguageSummary } from "@/core/bindings/types/xrf-translation";
import { CommandResult, ICommandResultStat } from "@/core/ui/command-result/CommandResult";
import { CommandResultFindings } from "@/core/ui/command-result/CommandResultFindings";
import { BaseComponentProps } from "@/lib/dom/element-types";

type TLanguageRow = TranslationVerifyLanguageSummary & { id: string; complete: number };

export interface ITranslationsVerifyResultProps extends BaseComponentProps {
  result: TranslationVerifySummary;
}

export function TranslationsVerifyResult({
  "data-testid": dataTestId = "translations-verify-result",
  id,
  className,
  sx,
  result,
}: ITranslationsVerifyResultProps): ReactElement {
  const columns: Array<GridColDef> = useMemo(
    () => [
      { field: "file", headerName: "File", flex: 1, minWidth: 260, cellClassName: "monospace" },
      { field: "language", headerName: "Language", width: 120, cellClassName: "monospace" },
      { field: "checked", headerName: "Ids", width: 100, type: "number" },
      { field: "missing", headerName: "Missing", width: 110, type: "number" },
      { field: "complete", headerName: "Complete", width: 120, valueFormatter: (value: number) => `${value}%` },
    ],
    []
  );

  const rows: Array<TLanguageRow> = useMemo(
    () =>
      result.languages.map((summary) => ({
        ...summary,
        id: `${summary.file}:${summary.language}`,
        // A file holding no ids is complete rather than divided by zero.
        complete:
          summary.checked === 0 ? 100 : Math.round(((summary.checked - summary.missing) / summary.checked) * 100),
      })),
    [result.languages]
  );

  // Which languages are incomplete, rather than which of 149,979 ids: the counts are what a row of the
  // table above is summarising, and the reason this screen reports the aggregate at all.
  const incompleteLanguages: Array<string> = useMemo(
    () =>
      Array.from(
        new Set(result.languages.filter((summary) => summary.missing > 0).map((summary) => summary.language))
      ).sort(),
    [result.languages]
  );

  const stats: Array<ICommandResultStat> = useMemo(
    () => [
      { label: "ids checked", value: result.checked },
      { label: "missing", value: result.missing, tone: result.missing ? "error" : "success" },
      { label: "files", value: new Set(result.languages.map((summary) => summary.file)).size },
      { label: "languages", value: new Set(result.languages.map((summary) => summary.language)).size },
      {
        label: "incomplete",
        value: incompleteLanguages.length ? incompleteLanguages.join(", ") : "none",
        tone: incompleteLanguages.length ? "error" : "success",
      },
    ],
    [result, incompleteLanguages]
  );

  return (
    <CommandResult
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={sx}
      headline={
        result.missing
          ? `${result.missing} translation(s) missing across ${incompleteLanguages.length} language(s)`
          : `Every language is complete across ${result.checked} id(s)`
      }
      tone={result.missing ? "error" : "success"}
      stats={stats}
    >
      <CommandResultFindings<TLanguageRow>
        rows={rows}
        columns={columns}
        getRowId={(row) => row.id}
        getSearchText={(row) => `${row.file} ${row.language}`}
        emptyLabel={"No translation sources were found to check."}
        searchPlaceholder={"Filter by file or language"}
      />
    </CommandResult>
  );
}
