import { Typography } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { cn, tid, uid } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ITexturePreviewFrameProps extends BaseComponentProps {
  /** What the picture is, or what is happening to it, on the bar beneath. */
  caption: string;
  /** Whether the content area draws the alpha checkerboard behind whatever it holds. */
  isCheckered?: boolean;
  children: ReactNode;
}

/**
 * The shell the preview keeps whatever it is showing.
 */
export function TexturePreviewFrame({
  "data-testid": dataTestId = "texture-preview-frame",
  id,
  className,
  caption,
  isCheckered = false,
  children,
}: ITexturePreviewFrameProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("flex min-h-0 min-w-0 grow flex-col", className)}>
      <div
        data-testid={tid(dataTestId, "body")}
        id={uid(id, "body")}
        className={cn("flex min-h-0 min-w-0 grow flex-col", isCheckered ? "checkerboard" : "bg-viewport-backdrop")}
      >
        {children}
      </div>

      <div
        data-testid={tid(dataTestId, "footer")}
        id={uid(id, "footer")}
        className={"shrink-0 border-t border-divider px-3 py-1"}
      >
        <Typography className={"text-text-secondary"} variant={"caption"} noWrap>
          {caption}
        </Typography>
      </div>
    </div>
  );
}
