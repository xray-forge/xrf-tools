import { Typography } from "@mui/material";
import { ReactElement } from "react";

import { PANEL } from "@/core/theme/tokens";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IEditorPanelEmptyProps extends BaseComponentProps {
  /** What is absent, and where it would come from once it is not. */
  label: string;
}

/**
 * Explains why a panel has no content to show.
 */
export function EditorPanelEmpty({
  "data-testid": dataTestId = "editor-panel-empty",
  id,
  className,
  label,
}: IEditorPanelEmptyProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("p-panel-content", className)}>
      <Typography variant={"body2"} sx={{ color: "text.secondary", lineHeight: PANEL.contentLineHeight }}>
        {label}
      </Typography>
    </div>
  );
}
