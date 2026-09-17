import { Divider, Typography } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IEditorPanelSectionProps extends BaseComponentProps {
  title: ReactNode;
  /** What distinguishes this group from a similar one beside it. */
  caption?: ReactNode;
  children: ReactNode;
  /** Suppresses the leading divider, so the first group does not draw one against the panel title. */
  isFirst?: boolean;
  /**
   * Takes the height the panel has left over, for content that scrolls on its own rather than flowing.
   */
  isFilling?: boolean;
}

/**
 * A titled group of rows.
 */
export function EditorPanelSection({
  "data-testid": dataTestId = "editor-panel-section",
  id,
  className,
  title,
  caption,
  children,
  isFirst,
  isFilling,
}: IEditorPanelSectionProps): ReactElement {
  return (
    <div
      data-testid={dataTestId}
      id={id}
      className={cn(
        "min-w-0 px-panel-content pb-panel-section",
        isFirst ? "pt-panel-content" : "pt-panel-section",
        isFilling ? "flex min-h-0 grow flex-col" : null,
        className
      )}
    >
      {isFirst ? null : <Divider className={"-mx-panel-content mb-panel-section"} />}

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
