import { Tooltip } from "@xrf/ui/material/tooltip";
import { Typography } from "@xrf/ui/material/typography";
import { ReactElement } from "react";

import { ApplicationStatusDetails } from "@/core/shell/status-bar/ApplicationStatusDetails";
import { TShellStatusSegment } from "@/core/shell/status-bar/shell-status-segment";

interface IApplicationStatusSegmentProps {
  segment: TShellStatusSegment;
}

/**
 * One segment of the status strip: a caption, or a caption whose hover lists its details.
 */
export function ApplicationStatusSegment({ segment }: IApplicationStatusSegmentProps): ReactElement {
  if (typeof segment === "string") {
    return (
      <Typography className={"text-text-secondary"} variant={"caption"} noWrap>
        {segment}
      </Typography>
    );
  }

  return (
    <Tooltip describeChild title={<ApplicationStatusDetails details={segment.details} />}>
      <Typography className={"text-text-secondary"} variant={"caption"} tabIndex={0} noWrap>
        {segment.text}
      </Typography>
    </Tooltip>
  );
}
