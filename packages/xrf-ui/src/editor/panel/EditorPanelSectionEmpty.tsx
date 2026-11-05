import { ReactElement } from "react";

import { BaseComponentProps } from "#/element-types";
import { Typography } from "#/material/typography";
import { cn } from "#/theme";

interface IEditorPanelSectionEmptyProps extends BaseComponentProps {
  /** What is absent from the section, and why: none in the file, or switched off elsewhere. */
  label: string;
}

/**
 * Explains why one section of a panel has nothing to show, in the type `EditorPanelEmpty` says it for a whole panel;
 * the section's own inset places it, so it adds none.
 */
export function EditorPanelSectionEmpty({
  "data-testid": dataTestId = "editor-panel-section-empty",
  id,
  className,
  label,
}: IEditorPanelSectionEmptyProps): ReactElement {
  return (
    <Typography
      data-testid={dataTestId}
      id={id}
      className={cn("block leading-panel text-text-secondary", className)}
      variant={"body2"}
    >
      {label}
    </Typography>
  );
}
