import { ReactElement, ReactNode } from "react";

import { BaseComponentProps } from "#/element-types";
import { DescriptionOutlinedIcon } from "#/icons/description-outlined";
import { CenteredColumn } from "#/layout/centered-column";
import { Typography } from "#/material/typography";
import { cn } from "#/theme";

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
    <CenteredColumn data-testid={dataTestId} id={id} className={cn("min-w-0 gap-2 p-6 text-center", className)}>
      <span aria-hidden={true} className={"flex [&>svg]:text-content-state-icon"}>
        {icon ?? <DescriptionOutlinedIcon className={"text-text-secondary opacity-55"} />}
      </span>

      <Typography variant={"subtitle1"}>{title}</Typography>

      <Typography className={"max-w-content-state-description wrap-anywhere text-text-secondary"} variant={"body2"}>
        {description}
      </Typography>

      {action}
    </CenteredColumn>
  );
}
