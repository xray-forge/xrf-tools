import { formatCount } from "@xrf/format";
import {
  TranslationFinding,
  TranslationVerifyLanguageSummary,
  TranslationVerifyResult,
} from "@xrf/ipc/types/xrf-translation";
import { CommandResult, ICommandResultStat } from "@xrf/ui/command-result/CommandResult";
import { CommandResultFindings } from "@xrf/ui/command-result/CommandResultFindings";
import { GridColDef } from "@xrf/ui/data-grid";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { identifierColumn, pathColumn } from "@xrf/ui/table/columns";
import { ReactElement, useMemo } from "react";

type TLanguageRow = TranslationVerifyLanguageSummary & { id: string; complete: number };

/**
 * One problem as a row, keyed by where it came in the run: two problems can say the same thing about the same file.
 */
type TProblemRow = TranslationFinding & { index: number };

/**
 * The run's verdict in one line, what the game would refuse or mangle before what it would show as an id.
 *
 * @param result - What the run found.
 * @param incomplete - How many languages miss at least one translation.
 * @returns The headline.
 */
function describeHeadline(result: TranslationVerifyResult, incomplete: number): string {
  if (result.problems.length) {
    return `${formatCount(result.problems.length)} problem(s) the game would refuse or show wrong`;
  }

  return result.missingTranslationsCount
    ? `${formatCount(result.missingTranslationsCount)} translation(s) missing across ${formatCount(incomplete)} language(s)`
    : `Every language is complete across ${formatCount(result.checkedTranslationsCount)} id(s)`;
}

export interface ITranslationsVerifyResultProps extends BaseComponentProps {
  result: TranslationVerifyResult;
}

export function TranslationsVerifyResult({
  "data-testid": dataTestId = "translations-verify-result",
  id,
  className,
  result,
}: ITranslationsVerifyResultProps): ReactElement {
  const columns: Array<GridColDef> = useMemo(
    () => [
      pathColumn("file", "File", 260),
      identifierColumn("language", "Language", 120),
      { field: "checked", headerName: "Ids", width: 100, type: "number" },
      { field: "missing", headerName: "Missing", width: 110, type: "number" },
      { field: "complete", headerName: "Complete", width: 120, valueFormatter: (value: number) => `${value}%` },
    ],
    []
  );

  const problemColumns: Array<GridColDef> = useMemo(
    () => [
      identifierColumn("rule", "Rule", 220),
      { field: "message", headerName: "Problem", flex: 1, minWidth: 260 },
      pathColumn("subject", "File", 220),
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

  const problems: Array<TProblemRow> = useMemo(
    () => result.problems.map((problem: TranslationFinding, index: number) => ({ ...problem, index })),
    [result.problems]
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
      { label: "ids checked", value: result.checkedTranslationsCount },
      {
        label: "missing",
        value: result.missingTranslationsCount,
        tone: result.missingTranslationsCount ? "error" : "success",
      },
      {
        label: "game problems",
        value: result.problems.length,
        tone: result.problems.length ? "error" : "success",
      },
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
      headline={describeHeadline(result, incompleteLanguages.length)}
      tone={result.problems.length || result.missingTranslationsCount ? "error" : "success"}
      stats={stats}
    >
      {problems.length ? (
        <CommandResultFindings<TProblemRow>
          rows={problems}
          columns={problemColumns}
          getRowId={(row) => row.index}
          getSearchText={(row) => `${row.rule} ${row.message} ${row.subject ?? ""}`}
          emptyLabel={"Nothing the game would refuse or mangle."}
          searchPlaceholder={"Filter by rule, problem or file"}
        />
      ) : null}

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
