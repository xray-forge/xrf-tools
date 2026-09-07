import { Box, Typography } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { mergeSx } from "@/core/theme/merge-sx";
import { PANEL } from "@/core/theme/tokens";
import { StyledComponentProps } from "@/lib/dom/element-types";

interface IEditorPanelHeaderProps extends StyledComponentProps {
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
      sx={mergeSx(
        {
          display: "flex",
          flexDirection: "column",
          gap: PANEL.sectionContentGap,
          paddingX: PANEL.contentPadding,
          paddingY: PANEL.headerPaddingY,
          flexShrink: 0,
          borderBottom: 1,
          borderColor: "divider",
        },
        sx
      )}
    >
      <Box sx={{ display: "flex", alignItems: "baseline", justifyContent: "space-between", gap: 1, minWidth: 0 }}>
        <Typography
          component={"h2"}
          variant={"subtitle2"}
          sx={{ color: "text.primary", overflowWrap: "anywhere", minWidth: 0 }}
        >
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
