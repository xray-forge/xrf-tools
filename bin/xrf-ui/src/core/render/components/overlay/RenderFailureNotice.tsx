import { default as ErrorOutlineIcon } from "@mui/icons-material/ErrorOutlineOutlined";
import { Typography } from "@mui/material";
import { ReactElement } from "react";

import { CenteredColumn } from "@/core/ui/layout/CenteredColumn";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IRenderFailureNoticeProps extends BaseComponentProps {
  /** Why the renderer stopped, as it said. */
  failure: string;
}

/**
 * Why a viewport draws nothing any more, in the viewport's own inks.
 */
export function RenderFailureNotice({
  "data-testid": dataTestId = "render-failure-notice",
  id,
  className,
  failure,
}: IRenderFailureNoticeProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("size-full text-viewport-text", className)} role={"alert"}>
      <CenteredColumn className={"p-6 text-center"}>
        <ErrorOutlineIcon aria-hidden={true} />

        <Typography variant={"subtitle1"}>The renderer stopped</Typography>

        <Typography className={"max-w-content-state-description wrap-anywhere"} variant={"body2"}>
          {failure}
        </Typography>
      </CenteredColumn>
    </div>
  );
}
