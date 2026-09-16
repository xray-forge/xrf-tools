import { Box } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { LAYOUT } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IArchiveDescriptionLayoutProps extends BaseComponentProps {
  children: ReactNode;
}

/**
 * The scrolling pane every format description is read in.
 */
export function ArchiveDescriptionLayout({
  "data-testid": dataTestId,
  id,
  className,
  children,
}: IArchiveDescriptionLayoutProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ flexGrow: 1, minWidth: 0, minHeight: 0, overflowY: "auto" }}
    >
      <Box sx={{ maxWidth: LAYOUT.readingColumnWidth, marginX: "auto" }}>{children}</Box>
    </Box>
  );
}
