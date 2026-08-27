import { Box, Typography } from "@mui/material";
import { ReactElement } from "react";

import { useEditorStatusSegments } from "@/core/shell/EditorStatusContext";
import { LAYOUT } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";

/**
 * Bottom status strip.
 */
export function ApplicationStatusBar({
  "data-testid": dataTestId = "application-status-bar",
  id = "application-status-bar",
  className,
}: BaseComponentProps): ReactElement {
  const segments: Array<string> = useEditorStatusSegments();

  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{
        display: "flex",
        alignItems: "center",
        justifyContent: "flex-end",
        gap: 1.5,
        height: LAYOUT.statusBarHeight,
        minHeight: LAYOUT.statusBarHeight,
        paddingX: 1,
        borderTop: 1,
        borderColor: "divider",
        backgroundColor: "background.paper",
      }}
    >
      {segments.length ? (
        segments.map((segment: string, index: number) => (
          <Typography key={segment + index} variant={"caption"} noWrap sx={{ color: "text.secondary" }}>
            {segment}
          </Typography>
        ))
      ) : (
        <Typography variant={"caption"} sx={{ color: "text.secondary", opacity: 0.7 }}>
          Ready
        </Typography>
      )}
    </Box>
  );
}
