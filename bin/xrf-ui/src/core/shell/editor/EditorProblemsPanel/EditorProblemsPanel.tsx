import { Box, Chip, List, ListItem, ListItemButton, Typography } from "@mui/material";
import { ReactElement } from "react";

import { EditorPanel, EditorPanelEmpty } from "@/core/shell/editor/EditorPanel";
import { MONOSPACE } from "@/core/theme/tokens";
import { splitLogicalPath } from "@/core/ui/tree/path-tree";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { LOGICAL_PATH_SEPARATOR } from "@/lib/path/separator";
import { Nullable, Optional } from "@/lib/types/general";

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
              </>
            );

            return (
              <ListItem
                key={`${finding.rule}-${finding.subject}-${index}`}
                data-testid={"editor-problems-row"}
                sx={{ display: "block" }}
                divider
                disablePadding={Boolean(location)}
              >
                {location ? (
                  <ListItemButton sx={{ display: "block" }} onClick={() => onSelect?.(location)}>
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
