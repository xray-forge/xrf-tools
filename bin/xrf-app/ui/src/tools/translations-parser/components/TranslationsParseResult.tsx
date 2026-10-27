import { Finding } from "@xrf/ipc/types/xrf-report";
import { TranslationParseResult } from "@xrf/ipc/types/xrf-translation";
import { Nullable } from "@xrf/types";
import { CommandResult, CommandResultFindings, ICommandResultStat, TCommandResultTone } from "@xrf/ui/command-result";
import { GridColDef } from "@xrf/ui/data-grid";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { identifierColumn, pathColumn } from "@xrf/ui/table";
import { ReactElement, useMemo } from "react";

import { EToolId } from "@/core/routing/tool";
import { RevealPathButton } from "@/core/ui/reveal/RevealPathButton";
import { listTranslationParseFindings } from "@/tools/translations-parser/lib/parse-findings";

export interface ITranslationsParseResultProps extends BaseComponentProps {
  result: TranslationParseResult;
  /** Where the sources were written, so the run can be opened; absent while nothing was written. */
  outputPath: Nullable<string>;
}

export function TranslationsParseResult({
  "data-testid": dataTestId = "translations-parse-result",
  id,
  className,
  result,
  outputPath,
}: ITranslationsParseResultProps): ReactElement {
  const columns: Array<GridColDef> = useMemo(
    () => [
      pathColumn("subject", "File", 280),
      identifierColumn("ruleId", "Rule", 220),
      { field: "message", headerName: "Detail", flex: 2, minWidth: 320 },
    ],
    []
  );

  const rows: Array<Finding & { id: number }> = useMemo(
    () => listTranslationParseFindings(result).map((finding: Finding, index: number) => ({ ...finding, id: index })),
    [result]
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

  // The census below counts files created and updated, which a preview did neither of.
  const headline: string = result.isDryRun
    ? `Preview only — would import ${result.census.entriesRead} entries as '${result.language}'`
    : `Imported ${result.census.entriesRead} entries as '${result.language}'`;

  // Findings outrank the mode, and a preview is neutral rather than a success.
  const tone: TCommandResultTone = rows.length > 0 ? "warning" : result.isDryRun ? "info" : "success";

  return (
    <CommandResult
      data-testid={dataTestId}
      id={id}
      className={className}
      headline={headline}
      tone={tone}
      stats={stats}
      actions={
        outputPath && !result.isDryRun ? (
          <RevealPathButton tool={EToolId.TRANSLATIONS_PARSER} path={outputPath} label={"Show output"} />
        ) : null
      }
    >
      <CommandResultFindings<Finding & { id: number }>
        rows={rows}
        columns={columns}
        getRowId={(row) => row.id}
        getSearchText={(row) => `${row.subject ?? ""} ${row.ruleId} ${row.message}`}
        emptyLabel={"Every string table was read without complaint."}
        searchPlaceholder={"Filter findings"}
      />
    </CommandResult>
  );
}
