import { Box, LinearProgress } from "@mui/material";
import { ReactElement, Ref } from "react";

import { ApplicationHelp } from "@/core/help/components/ApplicationHelp";
import { ApplicationTitleBarIcon } from "@/core/shell/title-bar/ApplicationTitleBarIcon";
import { WindowControls } from "@/core/shell/title-bar/WindowControls";
import { LAYOUT } from "@/core/theme/tokens";
import { DELAYED_REVEAL_SX } from "@/core/ui/layout/delayed-reveal";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IApplicationTitleBarProps extends BaseComponentProps {
  /** Where the active application portals its toolbar. The frame owns the element. */
  toolbarRef?: Ref<HTMLElement>;
  /** Draws the band's own progress line. Every editor already publishes this through `useEditorBusy`. */
  isBusy?: boolean;
}

/**
 * The window's single top band: drawn caption and the active application's toolbar in one row.
 */
export function ApplicationTitleBar({
  "data-testid": dataTestId = "application-title-bar",
  id = "application-title-bar",
  className,
  toolbarRef,
  isBusy,
}: IApplicationTitleBarProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      data-tauri-drag-region={"deep"}
      id={id}
      className={className}
      sx={{
        position: "relative",
        display: "flex",
        alignItems: "center",
        flexShrink: 0,
        height: LAYOUT.titleBarHeight,
        minHeight: LAYOUT.titleBarHeight,
        borderBottom: 1,
        borderColor: "divider",
        backgroundColor: "background.paper",
        userSelect: "none",
      }}
    >
      <ApplicationTitleBarIcon />

      <Box ref={toolbarRef} sx={{ display: "flex", alignItems: "center", flexGrow: 1, minWidth: 0, height: "100%" }} />

      <ApplicationHelp />

      <WindowControls />

      {isBusy ? (
        <LinearProgress sx={[DELAYED_REVEAL_SX, { position: "absolute", right: 0, bottom: 0, left: 0, height: 2 }]} />
      ) : null}
    </Box>
  );
}
