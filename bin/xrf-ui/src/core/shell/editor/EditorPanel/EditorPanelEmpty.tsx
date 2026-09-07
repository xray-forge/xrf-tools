import { Box, Typography } from "@mui/material";
import { ReactElement } from "react";

import { PANEL } from "@/core/theme/tokens";
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
    <Box data-testid={dataTestId} id={id} className={className} sx={{ padding: PANEL.contentPadding }}>
      <Typography variant={"body2"} sx={{ color: "text.secondary", lineHeight: PANEL.contentLineHeight }}>
        {label}
      </Typography>
    </Box>
  );
}
