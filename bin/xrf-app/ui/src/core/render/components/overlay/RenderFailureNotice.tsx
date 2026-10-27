import { BaseComponentProps } from "@xrf/ui/element-types";
import { ErrorOutlineOutlinedIcon as ErrorOutlineIcon } from "@xrf/ui/icons/error-outline-outlined";
import { CenteredColumn } from "@xrf/ui/layout/centered-column";
import { Typography } from "@xrf/ui/material/typography";
import { cn } from "@xrf/ui/theme";
import { ReactElement } from "react";

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
