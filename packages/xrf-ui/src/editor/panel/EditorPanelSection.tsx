import { ReactElement, ReactNode } from "react";

import { BaseComponentProps } from "#/element-types";
import { Typography } from "#/material/typography";
import { cn } from "#/theme";

interface IEditorPanelSectionProps extends BaseComponentProps {
  title: ReactNode;
  /** What distinguishes this group from a similar one beside it. */
  caption?: ReactNode;
  children: ReactNode;
  /**
   * Takes the height the panel has left over, for content that scrolls on its own rather than flowing.
   */
  isFilling?: boolean;
}

/**
 * A titled group of rows, laid out by the theme's `panel-section`: ruled off from a group directly before it, and at
 * the panel's content inset after anything else.
 */
export function EditorPanelSection({
  "data-testid": dataTestId = "editor-panel-section",
  id,
  className,
  title,
  caption,
  children,
  isFilling,
}: IEditorPanelSectionProps): ReactElement {
  return (
    <div
      data-testid={dataTestId}
      id={id}
      className={cn("panel-section", isFilling ? "flex min-h-0 grow flex-col" : null, className)}
    >
      <Typography className={"wrap-anywhere text-text-secondary"} component={"h3"} variant={"overline"}>
        {title}
      </Typography>

      {caption ? (
        <Typography className={"block wrap-anywhere text-text-disabled"} variant={"caption"}>
          {caption}
        </Typography>
      ) : null}

      <div className={cn("mt-panel-section-gap min-w-0", isFilling ? "min-h-0 grow" : null)}>{children}</div>
    </div>
  );
}
