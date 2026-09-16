import { Box, Typography } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { PanelCloseAction } from "@/core/shell/panel/PanelCloseAction";
import { mergeSx } from "@/core/theme/merge-sx";
import { LAYOUT, PANEL } from "@/core/theme/tokens";
import { StyledComponentProps } from "@/lib/dom/element-types";

interface IEditorPanelHeaderProps extends StyledComponentProps {
  /** What the panel holds, as its heading. */
  title: string;
  /** Stated opposite the title: a count, or whatever else the panel reports about itself. */
  caption?: ReactNode;
  /** Controls acting on the whole panel, at the end of the title row. */
  actions?: ReactNode;
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
  actions,
  children,
}: IEditorPanelHeaderProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={mergeSx({ display: "flex", flexDirection: "column", flexShrink: 0 }, sx)}
    >
      <Box
        sx={{
          display: "flex",
          alignItems: "center",
          minHeight: LAYOUT.headerHeight,
          paddingX: PANEL.contentPadding,
          borderBottom: 1,
          borderColor: "divider",
          backgroundColor: "background.frame",
        }}
      >
        <Box
          sx={{
            display: "flex",
            alignItems: "baseline",
            justifyContent: "space-between",
            gap: 1,
            minWidth: 0,
            flexGrow: 1,
          }}
        >
          <Typography
            component={"h2"}
            variant={"subtitle2"}
            sx={{ color: "text.primary", overflowWrap: "anywhere", minWidth: 0 }}
          >
            {title}
          </Typography>

          {caption ? (
            <Typography variant={"caption"} sx={{ color: "text.secondary" }}>
              {caption}
            </Typography>
          ) : null}
        </Box>

        <Box
          sx={{
            display: "flex",
            alignItems: "center",
            flexShrink: 0,
            marginLeft: 1,
          }}
        >
          {actions}

          <PanelCloseAction />
        </Box>
      </Box>

      {children ? (
        <Box
          sx={{
            display: "flex",
            flexDirection: "column",
            gap: PANEL.sectionContentGap,
            paddingX: PANEL.contentPadding,
            paddingY: PANEL.headerPaddingY,
            borderBottom: 1,
            borderColor: "divider",
          }}
        >
          {children}
        </Box>
      ) : null}
    </Box>
  );
}
