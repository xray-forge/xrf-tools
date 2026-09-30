import { Tooltip, Typography } from "@mui/material";
import { ReactElement } from "react";

import { TEditorStatusSegment } from "@/core/shell/editor-shell";
import { ApplicationStatusDetails } from "@/core/shell/footer/ApplicationStatusDetails";

interface IApplicationStatusSegmentProps {
  segment: TEditorStatusSegment;
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
