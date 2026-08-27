import { Box, Typography } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IEditorPanelHeaderProps extends BaseComponentProps {
  /** What the panel holds, as its heading. */
  title: string;
  /** Stated opposite the title: a count, or whatever else the panel reports about itself. */
  caption?: ReactNode;
  /** Controls belonging to the band rather than to the body, such as a filter field. */
  children?: ReactNode;
}

/**
 * Heading band of a docked panel.
 */
export function EditorPanelHeader({
  "data-testid": dataTestId = "editor-panel-header",
  id,
  className,
  sx,
  title,
  caption,
  children,
}: IEditorPanelHeaderProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={[
        {
          display: "flex",
          flexDirection: "column",
          gap: 1,
          padding: 1,
          borderBottom: 1,
          borderColor: "divider",
        },
        ...(sx === undefined ? [] : Array.isArray(sx) ? sx : [sx]),
      ]}
    >
      <Box sx={{ display: "flex", alignItems: "baseline", justifyContent: "space-between", gap: 1, paddingX: 0.5 }}>
        <Typography variant={"subtitle2"} sx={{ color: "text.primary" }}>
          {title}
        </Typography>

        {caption === undefined ? null : (
          <Typography variant={"caption"} sx={{ color: "text.secondary" }}>
            {caption}
          </Typography>
        )}
      </Box>

      {children}
    </Box>
  );
}
