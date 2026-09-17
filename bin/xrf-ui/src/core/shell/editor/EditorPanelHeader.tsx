import { Typography } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { PanelCloseAction } from "@/core/shell/panel/PanelCloseAction";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IEditorPanelHeaderProps extends BaseComponentProps {
  /** What the panel holds, as its heading. */
  title: string;
  /** Stated opposite the title: a count, or whatever else the panel reports about itself. */
  caption?: ReactNode;
  /** Controls acting on the whole panel, at the end of the title row. */
  actions?: ReactNode;
  /** Controls belonging to the band rather than to the body, such as a filter field. */
  children?: ReactNode;
}

/**
 * Heading band of a docked panel.
 */
export function EditorPanelHeader({
  "data-testid": dataTestId = "editor-panel-header",
  id,
  className,
  title,
  caption,
  actions,
  children,
}: IEditorPanelHeaderProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("flex shrink-0 flex-col", className)}>
      <div className={"flex min-h-header items-center border-b border-divider header-band px-panel-content"}>
        <div className={"flex min-w-0 grow items-baseline justify-between gap-2"}>
          <Typography className={"min-w-0 wrap-anywhere text-text-primary"} component={"h2"} variant={"subtitle2"}>
            {title}
          </Typography>

          {caption ? (
            <Typography className={"text-text-secondary"} variant={"caption"}>
              {caption}
            </Typography>
          ) : null}
        </div>

        <div className={"ml-2 flex shrink-0 items-center"}>
          {actions}

          <PanelCloseAction />
        </div>
      </div>

      {children ? (
        <div className={"flex flex-col gap-panel-section-gap border-b border-divider px-panel-content py-3"}>
          {children}
        </div>
      ) : null}
    </div>
  );
}
