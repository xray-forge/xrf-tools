import { ReactElement, useMemo } from "react";

import { CommandResult, ICommandResultStat, TCommandResultTone } from "@/core/ui/command-result/CommandResult";
import { CommandResultPathList } from "@/core/ui/command-result/CommandResultPathList";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatDuration } from "@/lib/format/duration";

/** What a formatting command over a tree answers, whichever format the tree holds. */
export interface IFormatCommandOutcome {
  totalFiles: number;
  validFiles: number;
  invalidFiles: number;
  /** Every item that was, or would be, rewritten. */
  toFormat: Array<string>;
  duration: number;
  startupDuration: number;
}

/** What the items a formatter reads are called, so one view reads right for each tool. */
export interface IFormatCommandNouns {
  /** One item, in lower case, such as `file`. */
  item: string;
  /** The column header over the list, such as `File`. */
  column: string;
  /** Every item at once, for the headline of a clean run, such as `files`. */
  everything: string;
}

interface IFormatCommandResultProps extends BaseComponentProps {
  isCheck: boolean;
  nouns: IFormatCommandNouns;
  result: IFormatCommandOutcome;
}

/**
 * How a formatting run over a tree went: what it read, what it found or rewrote, and each item it names.
 */
export function FormatCommandResult({
  "data-testid": dataTestId = "format-command-result",
  id,
  className,
  isCheck,
  nouns,
  result,
}: IFormatCommandResultProps): ReactElement {
  // In check mode an item that needs formatting is a failure; in write mode the same number is work done.
  const tone: TCommandResultTone = result.invalidFiles ? (isCheck ? "error" : "warning") : "success";

  const stats: Array<ICommandResultStat> = useMemo(
    () => [
      { label: `${nouns.item}s`, value: result.totalFiles },
      { label: "valid", value: result.validFiles, tone: "success" },
      { label: isCheck ? "need formatting" : "formatted", value: result.invalidFiles, tone },
      { label: "elapsed", value: formatDuration(result.duration) },
      { label: "opening", value: formatDuration(result.startupDuration) },
    ],
    [isCheck, nouns.item, result, tone]
  );

  return (
    <CommandResult
      data-testid={dataTestId}
      id={id}
      className={className}
      headline={
        result.invalidFiles
          ? isCheck
            ? `${result.invalidFiles} ${nouns.item}(s) are not correctly formatted`
            : `Formatted ${result.invalidFiles} ${nouns.item}(s)`
          : `All ${nouns.everything} are correctly formatted`
      }
      tone={tone}
      stats={stats}
    >
      <CommandResultPathList
        paths={result.toFormat}
        column={nouns.column}
        emptyLabel={"Nothing to format."}
        searchPlaceholder={`Filter by ${nouns.item}`}
      />
    </CommandResult>
  );
}
