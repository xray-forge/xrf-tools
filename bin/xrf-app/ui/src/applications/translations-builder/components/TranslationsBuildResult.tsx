import { TranslationBuildLanguageSummary, TranslationBuildResult } from "@xrf/ipc/types/xrf-translation";
import { Nullable } from "@xrf/types";
import { CommandResult, CommandResultFindings, ICommandResultStat } from "@xrf/ui/command-result";
import { GridColDef } from "@xrf/ui/data-grid";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { identifierColumn } from "@xrf/ui/table";
import { ReactElement, useMemo } from "react";

import { EApplicationId } from "@/core/routing/application";
import { RevealPathButton } from "@/core/ui/reveal/RevealPathButton";

type TLanguageRow = TranslationBuildLanguageSummary & { id: string };

export interface ITranslationsBuildResultProps extends BaseComponentProps {
  result: TranslationBuildResult;
  /** Where the build was told to write; the summary does not carry an address. */
  outputPath: Nullable<string>;
}

export function TranslationsBuildResult({
  "data-testid": dataTestId = "translations-build-result",
  id,
  className,
  result,
  outputPath,
}: ITranslationsBuildResultProps): ReactElement {
  const columns: Array<GridColDef> = useMemo(
    () => [
      identifierColumn("language", "Language", 140),
      { field: "files", headerName: "String tables", width: 150, type: "number" },
      { field: "entries", headerName: "Ids compiled", width: 150, type: "number" },
    ],
    []
  );

  const rows: Array<TLanguageRow> = useMemo(
    () => result.languages.map((summary) => ({ ...summary, id: summary.language })),
    [result.languages]
  );

  const stats: Array<ICommandResultStat> = useMemo(
    () => [
      { label: "sources", value: result.sources },
      { label: "string tables", value: result.files },
      { label: "languages", value: result.languages.length },
    ],
    [result]
  );

  return (
    <CommandResult
      data-testid={dataTestId}
      id={id}
      className={className}
      headline={`Built ${result.files} string table(s) from ${result.sources} source(s)`}
      tone={"success"}
      stats={stats}
      actions={
        <RevealPathButton application={EApplicationId.TRANSLATIONS_BUILDER} path={outputPath} label={"Show output"} />
      }
    >
      <CommandResultFindings<TLanguageRow>
        rows={rows}
        columns={columns}
        getRowId={(row) => row.id}
        getSearchText={(row) => row.language}
        emptyLabel={"No translation sources were found to build."}
        searchPlaceholder={"Filter by language"}
      />
    </CommandResult>
  );
}
