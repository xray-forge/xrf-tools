import { ReactElement, ReactNode } from "react";

import { BaseComponentProps } from "#/element-types";
import { DescriptionOutlinedIcon } from "#/icons/description-outlined";
import { CenteredColumn } from "#/layout/centered-column";
import { cn } from "#/theme";
import { Typography } from "#/typography";

interface IEmptyStateProps extends BaseComponentProps {
  title: string;
  description: string;
  /** Overrides the default document glyph where a surface has a better one. */
  icon?: ReactNode;
  /** The way out, for a dead end the surface can undo. Omit where there is nothing to offer. */
  action?: ReactNode;
  /** Draws in the viewport's inks, for a state shown over a scene rather than on a panel. */
  isOnViewport?: boolean;
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
  isOnViewport = false,
}: IEmptyStateProps): ReactElement {
  return (
    <CenteredColumn
      data-testid={dataTestId}
      id={id}
      className={cn("min-w-0 gap-2 p-6 text-center", isOnViewport ? "text-viewport-text" : null, className)}
    >
      <span aria-hidden={true} className={"flex [&>svg]:text-content-state-icon"}>
        {icon ?? <DescriptionOutlinedIcon className={cn("opacity-55", isOnViewport ? null : "text-text-secondary")} />}
      </span>

      <Typography variant={"subtitle1"}>{title}</Typography>

      <Typography
        className={cn(
          "max-w-content-state-description wrap-anywhere whitespace-pre-line",
          isOnViewport ? "text-viewport-text" : "text-text-secondary"
        )}
        variant={"body2"}
      >
        {description}
      </Typography>

      {action}
    </CenteredColumn>
  );
}
