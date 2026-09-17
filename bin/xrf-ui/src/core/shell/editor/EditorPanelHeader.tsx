import { Box, Typography } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { PanelCloseAction } from "@/core/shell/panel/PanelCloseAction";
import { getHeaderBandSx } from "@/core/theme/surface";
import { cn } from "@/lib/dom/dom-name";
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
    <Box data-testid={dataTestId} id={id} className={cn("flex shrink-0 flex-col", className)} sx={sx}>
      <Box className={"flex min-h-header items-center border-b border-divider px-panel-content"} sx={getHeaderBandSx}>
        <div className={"flex min-w-0 grow items-baseline justify-between gap-2"}>
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
        </div>

        <div className={"ml-2 flex shrink-0 items-center"}>
          {actions}

          <PanelCloseAction />
        </div>
      </Box>

      {children ? (
        <div className={"flex flex-col gap-panel-section-gap border-b border-divider px-panel-content py-3"}>
          {children}
        </div>
      ) : null}
    </Box>
  );
}
