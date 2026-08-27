import { GridColDef } from "@mui/x-data-grid";
import { ReactElement, useMemo } from "react";

import { TranslationParseFinding, TranslationParseSummary } from "@/core/bindings/types/xrf-app";
import { EApplicationId } from "@/core/routing/application";
import { CommandResult, ICommandResultStat } from "@/core/ui/command-result/CommandResult";
import { CommandResultFindings } from "@/core/ui/command-result/CommandResultFindings";
import { RevealPathButton } from "@/core/ui/reveal/RevealPathButton";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

export interface ITranslationsParseResultProps extends BaseComponentProps {
  result: TranslationParseSummary;
  /** Where the sources were written, so the run can be opened; absent while nothing was written. */
  outputPath: Nullable<string>;
}

export function TranslationsParseResult({
  "data-testid": dataTestId = "translations-parse-result",
  id,
  className,
  sx,
  result,
  outputPath,
}: ITranslationsParseResultProps): ReactElement {
  const columns: Array<GridColDef> = useMemo(
    () => [
      { field: "subject", headerName: "File", flex: 1, minWidth: 280, cellClassName: "monospace" },
      { field: "rule", headerName: "Rule", width: 220, cellClassName: "monospace" },
      { field: "message", headerName: "Detail", flex: 2, minWidth: 320 },
    ],
    []
  );

  const rows: Array<TranslationParseFinding & { id: number }> = useMemo(
    () => result.findings.map((finding, index) => ({ ...finding, id: index })),
    [result.findings]
  );

  const stats: Array<ICommandResultStat> = useMemo(
    () => [
      { label: "files read", value: result.census.filesRead },
      { label: "created", value: result.census.filesCreated },
      { label: "updated", value: result.census.filesUpdated },
      { label: "unchanged", value: result.census.filesUnchanged },
      { label: "inserted", value: result.census.entriesInserted },
      { label: "filled", value: result.census.entriesFilled },
      { label: "placeholders", value: result.census.placeholdersAdded },
      { label: "conflicts", value: result.census.entriesConflicted },
    ],
    [result.census]
  );

  const headline: string = result.isDryRun
    ? `Would import ${result.census.entriesRead} entries as '${result.language}'`
    : `Imported ${result.census.entriesRead} entries as '${result.language}'`;

  return (
    <CommandResult
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={sx}
      headline={headline}
      tone={result.findings.length > 0 ? "warning" : "success"}
      stats={stats}
      actions={
        outputPath && !result.isDryRun ? (
          <RevealPathButton application={EApplicationId.TRANSLATIONS_PARSER} path={outputPath} label={"Show output"} />
        ) : null
      }
    >
      <CommandResultFindings<TranslationParseFinding & { id: number }>
        rows={rows}
        columns={columns}
        getRowId={(row) => row.id}
        getSearchText={(row) => `${row.subject ?? ""} ${row.rule} ${row.message}`}
        emptyLabel={"Every string table was read without complaint."}
        searchPlaceholder={"Filter findings"}
      />
    </CommandResult>
  );
}
