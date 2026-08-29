import { GridColDef } from "@mui/x-data-grid";
import { ReactElement, useMemo } from "react";

import { TranslationBuildSummary } from "@/core/bindings/types/xrf-app";
import { ProjectBuildLanguageSummary } from "@/core/bindings/types/xrf-translation";
import { EApplicationId } from "@/core/routing/application";
import { CommandResult, ICommandResultStat } from "@/core/ui/command-result/CommandResult";
import { CommandResultFindings } from "@/core/ui/command-result/CommandResultFindings";
import { RevealPathButton } from "@/core/ui/reveal/RevealPathButton";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

type TLanguageRow = ProjectBuildLanguageSummary & { id: string };

export interface ITranslationsBuildResultProps extends BaseComponentProps {
  result: TranslationBuildSummary;
  /** Where the build was told to write; the summary does not carry an address. */
  outputPath: Nullable<string>;
}

export function TranslationsBuildResult({
  "data-testid": dataTestId = "translations-build-result",
  id,
  className,
  sx,
  result,
  outputPath,
}: ITranslationsBuildResultProps): ReactElement {
  const columns: Array<GridColDef> = useMemo(
    () => [
      { field: "language", headerName: "Language", width: 140, cellClassName: "monospace" },
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
      sx={sx}
      headline={`Built ${result.files} string table(s) from ${result.sources} source(s)`}
      tone={"success"}
      stats={stats}
      actions={
        <RevealPathButton
          application={EApplicationId.TRANSLATIONS_BUILDER}
          path={outputPath}
          label={"Show output"}
        />
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
