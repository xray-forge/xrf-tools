import { default as DescriptionOutlinedIcon } from "@mui/icons-material/DescriptionOutlined";
import { Box, Typography } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { CONTENT_STATE } from "@/core/theme/tokens";
import { CenteredColumn } from "@/core/ui/layout/CenteredColumn";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IEmptyStateProps extends BaseComponentProps {
  title: string;
  description: string;
  /** Overrides the default document glyph where a surface has a better one. */
  icon?: ReactNode;
  /** The way out, for a dead end the surface can undo. Omit where there is nothing to offer. */
  action?: ReactNode;
}

/**
 * What a surface shows when it has nothing to show.
 */
export function EmptyState({
  "data-testid": dataTestId,
  id,
  className,
  action,
  description,
  icon,
  title,
}: IEmptyStateProps): ReactElement {
  return (
    <CenteredColumn
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ padding: CONTENT_STATE.padding, gap: CONTENT_STATE.gap, minWidth: 0, textAlign: "center" }}
    >
      <Box aria-hidden={true} sx={{ display: "flex", "& .MuiSvgIcon-root": { fontSize: CONTENT_STATE.iconSize } }}>
        {icon ?? <DescriptionOutlinedIcon sx={{ color: "text.secondary", opacity: 0.55 }} />}
      </Box>

      <Typography variant={"subtitle1"}>{title}</Typography>

      <Typography
        variant={"body2"}
        sx={{ maxWidth: CONTENT_STATE.descriptionMaxWidth, color: "text.secondary", overflowWrap: "anywhere" }}
      >
        {description}
      </Typography>

      {action}
    </CenteredColumn>
  );
}
