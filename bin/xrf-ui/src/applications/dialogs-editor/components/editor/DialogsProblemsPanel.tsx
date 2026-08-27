import { Box, Chip, List, ListItem, ListItemText, Typography } from "@mui/material";
import { ReactElement } from "react";

import { DialogFinding } from "@/core/bindings/types/xrf-dialog";
import { EmptyState } from "@/core/ui/layout/EmptyState";

/**
 * Trims the directory off a subject, which is otherwise most of the line.
 *
 * Subjects here are logical paths, so the separator is the engine one and not the host platform's.
 */
function toLeaf(subject: string): string {
  const separator: number = subject.lastIndexOf("\\");

  return separator === -1 ? subject : subject.slice(separator + 1);
}

export interface IDialogsProblemsPanelProps {
  findings: ReadonlyArray<DialogFinding>;
}

/**
 * What the reader found but did not refuse to open for.
 */
export function DialogsProblemsPanel({ findings }: IDialogsProblemsPanelProps): ReactElement {
  return findings.length ? (
    <Box sx={{ width: "100%", height: "100%", overflowY: "auto" }}>
      <List dense disablePadding>
        {findings.map((finding: DialogFinding, index: number) => (
          <ListItem key={`${finding.rule}-${finding.subject}-${index}`} alignItems={"flex-start"} divider>
            <ListItemText
              primary={
                <Box sx={{ display: "flex", alignItems: "center", gap: 0.75, minWidth: 0 }}>
                  <Chip size={"small"} variant={"outlined"} label={finding.rule.replace("dialog.", "")} />

                  {finding.subject ? (
                    <Typography variant={"caption"} noWrap sx={{ color: "text.secondary" }} title={finding.subject}>
                      {toLeaf(finding.subject)}
                    </Typography>
                  ) : null}
                </Box>
              }
              secondary={finding.message}
            />
          </ListItem>
        ))}
      </List>
    </Box>
  ) : (
    <EmptyState
      title={"No problems found"}
      description={"Every dialog and string table in this project read cleanly."}
    />
  );
}
