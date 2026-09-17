import { Box } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { cn } from "@/lib/dom/dom-name";
import { StyledComponentProps } from "@/lib/dom/element-types";

interface ICenteredColumnProps extends StyledComponentProps {
  children: ReactNode;
}

/**
 * Full-size flex column that centers its children both axes.
 */
export function CenteredColumn({
  "data-testid": dataTestId,
  id,
  className,
  children,
  sx,
}: ICenteredColumnProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={cn("flex h-full w-full flex-col items-center justify-center gap-2", className)}
      sx={sx}
    >
      {children}
    </Box>
  );
}
