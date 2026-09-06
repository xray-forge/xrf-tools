import { Box, Chip, List, ListItem, Typography } from "@mui/material";
import { ReactElement } from "react";

import { EditorPanel, EditorPanelEmpty } from "@/core/shell/editor/EditorPanel";
import { MONOSPACE } from "@/core/theme/tokens";
import { LOGICAL_PATH_SEPARATOR, splitLogicalPath } from "@/core/ui/tree/path-tree";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

/** A problem reported while reading an editor project. */
export interface IEditorProblem {
  rule: string;
  subject: Nullable<string>;
  message: string;
}

interface IEditorProblemsPanelProps extends BaseComponentProps {
  findings: ReadonlyArray<IEditorProblem>;
  /** Namespace to omit from the start of rule labels, including its separator. */
  rulePrefix?: string;
  /** Explains what was checked when no problems were found. */
  emptyDescription: string;
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

            return (
              <ListItem key={`${finding.rule}-${finding.subject}-${index}`} sx={{ display: "block" }} divider>
                <Box sx={{ display: "flex", alignItems: "center", flexWrap: "wrap", gap: 0.75, minWidth: 0 }}>
                  <Chip
                    size={"small"}
                    variant={"outlined"}
                    label={rule}
                    title={finding.rule}
                    sx={{ maxWidth: "100%" }}
                  />

                  {subject ? (
                    <Typography
                      component={"span"}
                      variant={"caption"}
                      title={finding.subject ?? undefined}
                      sx={{ ...MONOSPACE, color: "text.secondary", minWidth: 0, overflowWrap: "anywhere" }}
                    >
                      {subject}
                    </Typography>
                  ) : null}
                </Box>

                <Typography
                  variant={"body2"}
                  sx={{ color: "text.secondary", marginTop: 0.5, overflowWrap: "anywhere" }}
                >
                  {finding.message}
                </Typography>
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
