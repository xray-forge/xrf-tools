import { Box } from "@mui/material";
import { ReactElement } from "react";

import { BaseComponentProps } from "@/lib/dom/element-types";

/**
 * The one vertical rule of a toolbar, between control groups and between the application and the window.
 */
export function EditorToolbarSeparator({
  "data-testid": dataTestId = "editor-toolbar-separator",
  id,
  className,
}: BaseComponentProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      aria-hidden={true}
      id={id}
      className={className}
      sx={{ width: "1px", height: 18, marginX: 0.5, flexShrink: 0, backgroundColor: "divider" }}
    />
  );
}
