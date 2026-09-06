import { Box, Typography } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { MONOSPACE } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IEditorPanelRowProps extends BaseComponentProps {
  label: string;
  value: ReactNode;
  /** Places long values below their labels instead of beside them. */
  isStacked?: boolean;
  /** Uses the shared identifier and path typography for the value. */
  isMonospace?: boolean;
}

/**
 * One label and value pair.
 *
 * The value is allowed to wrap onto its own line, because a path or identifier can be longer than the
 * panel is wide and truncating it would hide the part that identifies it.
 */
export function EditorPanelRow({
  "data-testid": dataTestId = "visual-panel-row",
  id,
  className,
  label,
  value,
  isStacked = false,
  isMonospace = false,
}: IEditorPanelRowProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{
        display: "flex",
        flexDirection: isStacked ? "column" : "row",
        justifyContent: "space-between",
        alignItems: "baseline",
        gap: isStacked ? 0.25 : 2,
        paddingY: isStacked ? 0.75 : 0.4,
        minWidth: 0,
        lineHeight: 1.6,
      }}
    >
      <Typography
        variant={isStacked ? "caption" : "body2"}
        sx={{ color: "text.secondary", flexShrink: 0, maxWidth: "100%", overflowWrap: "anywhere" }}
      >
        {label}
      </Typography>

      <Typography
        component={"span"}
        variant={"body2"}
        sx={{
          ...(isMonospace ? MONOSPACE : null),
          minWidth: 0,
          maxWidth: "100%",
          textAlign: isStacked ? "left" : "right",
          overflowWrap: "anywhere",
        }}
      >
        {value}
      </Typography>
    </Box>
  );
}
