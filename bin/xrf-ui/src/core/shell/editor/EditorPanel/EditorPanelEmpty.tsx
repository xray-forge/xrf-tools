import { Typography } from "@mui/material";
import { ReactElement } from "react";

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
      <Typography className={"leading-panel text-text-secondary"} variant={"body2"}>
        {label}
      </Typography>
    </div>
  );
}
