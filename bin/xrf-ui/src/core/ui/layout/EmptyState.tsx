import { default as DescriptionOutlinedIcon } from "@mui/icons-material/DescriptionOutlined";
import { Typography } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { CenteredColumn } from "@/core/ui/layout/CenteredColumn";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IEmptyStateProps extends BaseComponentProps {
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
export function EmptyState({ action, description, icon, title }: IEmptyStateProps): ReactElement {
  return (
    <CenteredColumn sx={{ padding: 3, textAlign: "center" }}>
      {icon ?? <DescriptionOutlinedIcon sx={{ fontSize: 40, color: "text.secondary", opacity: 0.55 }} />}

      <Typography variant={"subtitle1"}>{title}</Typography>

      <Typography variant={"body2"} sx={{ maxWidth: 440, color: "text.secondary" }}>
        {description}
      </Typography>

      {action}
    </CenteredColumn>
  );
}
