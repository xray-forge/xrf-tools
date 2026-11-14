import { formatCount } from "@xrf/format";
import { Nullable } from "@xrf/types";
import { WarningAmberIcon } from "@xrf/ui/icons/warning-amber";
import { Tooltip } from "@xrf/ui/material/tooltip";
import { TreeRowLabel } from "@xrf/ui/tree";
import { ReactElement } from "react";

interface ILevelEditorOutlinerToolLabelProps {
  label: string;
  /** What the tool holds, said beside its label; none before it is read. */
  caption: Nullable<string>;
  /** How many of the scene's problems are the tool's. */
  problems: number;
}

/**
 * A tool's row in the Outliner: its name, what it holds, and a mark where the Problems panel lists something of it.
 */
export function LevelEditorOutlinerToolLabel({
  label,
  caption,
  problems,
}: ILevelEditorOutlinerToolLabelProps): ReactElement {
  const warning: string =
    problems === 1 ? "1 problem, listed in Problems" : `${formatCount(problems)} problems, listed in Problems`;

  return (
    <span className={"flex min-w-0 items-center gap-1"}>
      <TreeRowLabel label={label} caption={caption} captionTitle={caption ?? undefined} />

      {problems ? (
        <Tooltip title={warning}>
          <WarningAmberIcon aria-label={warning} className={"shrink-0 text-warning"} fontSize={"inherit"} />
        </Tooltip>
      ) : null}
    </span>
  );
}
