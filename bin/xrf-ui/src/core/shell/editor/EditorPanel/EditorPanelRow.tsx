import { Box, Typography } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { BaseComponentProps } from "@/lib/dom/element-types";

interface IEditorPanelRowProps extends BaseComponentProps {
  label: string;
  value: ReactNode;
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
}: IEditorPanelRowProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{
        display: "flex",
        justifyContent: "space-between",
        alignItems: "baseline",
        gap: 2,
        paddingY: 0.4,
        lineHeight: 1.6,
      }}
    >
      <Typography variant={"body2"} sx={{ color: "text.secondary", flexShrink: 0 }}>
        {label}
      </Typography>

      <Typography component={"span"} variant={"body2"} sx={{ textAlign: "right", wordBreak: "break-all" }}>
        {value}
      </Typography>
    </Box>
  );
}
