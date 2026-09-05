import { Box, Typography } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { mergeSx } from "@/core/theme/merge-sx";
import { StyledComponentProps } from "@/lib/dom/element-types";

interface IEditorPanelProps extends StyledComponentProps {
  /** The name this panel's stripe button carries. */
  title: string;
  children: ReactNode;
}

/**
 * One panel, titled with the name its stripe button carries.
 */
export function EditorPanel({
  "data-testid": dataTestId = "visual-panel",
  id,
  className,
  sx,
  title,
  children,
}: IEditorPanelProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={mergeSx({ display: "flex", flexDirection: "column", minHeight: 0 }, sx)}
    >
      <Box sx={{ paddingX: 2, paddingY: 1.5, borderBottom: 1, borderColor: "divider" }}>
        <Typography variant={"subtitle1"}>{title}</Typography>
      </Box>

      <Box sx={{ flexGrow: 1, minHeight: 0 }}>{children}</Box>
    </Box>
  );
}
