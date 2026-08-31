import { Box } from "@mui/material";
import { ReactElement } from "react";

import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IXrfMarkProps extends BaseComponentProps {
  /** Rendered edge in pixels. The drawing is tuned for small sizes; see the note above. */
  size?: number;
  /** Names the mark for assistive technology. Omit where a label already sits beside it. */
  title?: string;
}

export function XrfMark({
  "data-testid": dataTestId = "xrf-mark",
  id,
  className,
  sx,
  size = 16,
  title,
}: IXrfMarkProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      aria-label={title}
      aria-hidden={title === undefined || undefined}
      id={id}
      className={className}
      component={"svg"}
      viewBox={"0 0 256 256"}
      role={"img"}
      sx={[{ width: size, height: size, flexShrink: 0 }, ...(Array.isArray(sx) ? sx : [sx])]}
    >
      <path
        d={
          "M 128 128 L 182.25 34.036 A 108.5 108.5 0 0 0 73.75 34.036 Z " +
          "M 128 128 L 19.5 128 A 108.5 108.5 0 0 0 73.75 221.964 Z " +
          "M 128 128 L 182.25 221.964 A 108.5 108.5 0 0 0 236.5 128 Z"
        }
        fill={"currentColor"}
      />
      <circle cx={128} cy={128} r={116} fill={"none"} stroke={"currentColor"} strokeWidth={18} />
      <polygon points={"161 128 144.5 99.421 111.5 99.421 95 128 111.5 156.579 144.5 156.579"} fill={"#FFA200"} />
    </Box>
  );
}
