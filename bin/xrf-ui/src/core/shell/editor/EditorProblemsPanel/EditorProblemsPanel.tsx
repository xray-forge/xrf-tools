import { Chip, List, ListItem, ListItemButton, Typography } from "@mui/material";
import { Nullable, Optional } from "@xrf/types";
import { ReactElement } from "react";

import { EditorPanel, EditorPanelEmpty } from "@/core/shell/editor/EditorPanel";
import { splitLogicalPath } from "@/core/ui/tree/path-tree";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { LOGICAL_PATH_SEPARATOR } from "@/lib/path/separator";

/** Where a problem can be opened, for a surface that can take a reader there. */
export interface IEditorProblemLocation {
  /** Engine identity of the file to open. */
  path: string;
  /** One-based line inside it, where the finding reached one. */
  line: Nullable<number>;
}

/** A problem reported while reading an editor project. */
export interface IEditorProblem {
  rule: string;
  subject: Nullable<string>;
  message: string;
  /**
   * Where opening this problem would take a reader, when anything can.
   */
  location?: IEditorProblemLocation;
}

interface IEditorProblemsPanelProps extends BaseComponentProps {
  findings: ReadonlyArray<IEditorProblem>;
  /** Namespace to omit from the start of rule labels, including its separator. */
  rulePrefix?: string;
  /** Explains what was checked when no problems were found. */
  emptyDescription: string;
  /** A finding naming a place was chosen. Rows without a location never call it. */
  onSelect?: (location: IEditorProblemLocation) => void;
}

/**
 * Project findings with compact subjects and the original identifiers available in tooltips.
 */
export function EditorProblemsPanel({
  "data-testid": dataTestId = "editor-problems-panel",
  id,
  className,
  findings,
  rulePrefix = "",
  emptyDescription,
  onSelect,
}: IEditorProblemsPanelProps): ReactElement {
  return (
    <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Problems"}>
      {findings.length ? (
        <List dense disablePadding>
          {findings.map((finding: IEditorProblem, index: number) => {
            const rule: string = finding.rule.startsWith(rulePrefix)
              ? finding.rule.slice(rulePrefix.length)
              : finding.rule;
            const subject: Nullable<string> = finding.subject
              ? splitLogicalPath(finding.subject.replaceAll("/", LOGICAL_PATH_SEPARATOR)).name || finding.subject
              : null;

            const location: Optional<IEditorProblemLocation> = onSelect ? finding.location : undefined;
            const content: ReactElement = (
              <>
                <div className={"flex min-w-0 flex-wrap items-center gap-1.5"}>
                  <Chip
                    className={"max-w-full"}
                    size={"small"}
                    variant={"outlined"}
                    label={rule}
                    title={finding.rule}
                  />

                  {subject ? (
                    <Typography
                      className={"monospace min-w-0 wrap-anywhere text-text-secondary"}
                      component={"span"}
                      variant={"caption"}
                      title={finding.subject ?? undefined}
                    >
                      {subject}
                    </Typography>
                  ) : null}
                </div>

                <Typography className={"mt-1 wrap-anywhere text-text-secondary"} variant={"body2"}>
                  {finding.message}
                </Typography>
              </>
            );

            return (
              <ListItem
                key={`${finding.rule}-${finding.subject}-${index}`}
                data-testid={"editor-problems-row"}
                className={"block"}
                divider
                disablePadding={Boolean(location)}
              >
                {location ? (
                  <ListItemButton className={"block"} onClick={() => onSelect?.(location)}>
                    {content}
                  </ListItemButton>
                ) : (
                  content
                )}
              </ListItem>
            );
          })}
        </List>
      ) : (
        <EditorPanelEmpty label={`No problems found. ${emptyDescription}`} />
      )}
    </EditorPanel>
  );
}
