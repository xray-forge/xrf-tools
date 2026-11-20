import { ReactElement, ReactNode } from "react";

import { BaseComponentProps } from "#/element-types";
import { cn } from "#/theme";
import { Typography } from "#/typography";

export interface IPreviewFrameProps extends BaseComponentProps {
  /** What the picture is, or what is happening to it, on the bar beneath. */
  caption: string;
  /** Whether the content area draws the alpha checkerboard behind whatever it holds. */
  isCheckered?: boolean;
  children: ReactNode;
}

/**
 * The shell a preview keeps whatever it is showing in: a viewport body on its backdrop or the alpha checkerboard, and
 * a caption bar beneath.
 */
export function PreviewFrame({
  "data-testid": dataTestId = "preview-frame",
  id,
  className,
  caption,
  isCheckered = false,
  children,
}: IPreviewFrameProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("flex min-h-0 min-w-0 grow flex-col", className)}>
      <div
        data-testid={`${dataTestId}-body`}
        id={id ? `${id}-body` : undefined}
        className={cn("flex min-h-0 min-w-0 grow flex-col", isCheckered ? "checkerboard" : "bg-viewport-backdrop")}
      >
        {children}
      </div>

      <div
        data-testid={`${dataTestId}-footer`}
        id={id ? `${id}-footer` : undefined}
        className={"shrink-0 border-t border-divider px-3 py-1"}
      >
        <Typography className={"text-text-secondary"} variant={"caption"} noWrap>
          {caption}
        </Typography>
      </div>
    </div>
  );
}
