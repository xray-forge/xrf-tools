import { Box, Divider, Typography } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IVisualPanelSectionProps extends BaseComponentProps {
  title: ReactNode;
  /** What distinguishes this group from a similar one beside it. */
  caption?: ReactNode;
  children: ReactNode;
  /** Suppresses the leading divider, so the first group does not draw one against the panel title. */
  isFirst?: boolean;
}

/**
 * A titled group of rows.
 */
export function VisualPanelSection({
  "data-testid": dataTestId = "visual-panel-section",
  id,
  className,
  title,
  caption,
  children,
  isFirst,
}: IVisualPanelSectionProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ paddingX: 2, paddingTop: isFirst ? 2 : 1.5, paddingBottom: 1.5 }}
    >
      {isFirst ? null : <Divider sx={{ marginBottom: 1.5, marginX: -2 }} />}

      <Typography variant={"overline"} sx={{ color: "text.secondary" }}>
        {title}
      </Typography>

      {caption ? (
        <Typography variant={"caption"} sx={{ display: "block", color: "text.disabled", overflowWrap: "anywhere" }}>
          {caption}
        </Typography>
      ) : null}

      <Box sx={{ marginTop: 1 }}>{children}</Box>
    </Box>
  );
}
