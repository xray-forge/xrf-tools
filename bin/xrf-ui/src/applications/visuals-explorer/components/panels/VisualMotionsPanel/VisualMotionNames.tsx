import { Typography } from "@mui/material";
import { ReactElement } from "react";

import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IVisualMotionNamesProps extends BaseComponentProps {
  names: Array<string>;
}

/** A list of motion names, wrapped rather than truncated because the tail identifies them. */
export function VisualMotionNames({
  "data-testid": dataTestId = "visual-motion-names",
  id,
  className,
  names,
}: IVisualMotionNamesProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={className}>
      {names.map((name) => (
        <Typography key={name} className={"leading-panel break-all"} variant={"body2"} sx={{ paddingY: 0.4 }}>
          {name}
        </Typography>
      ))}
    </div>
  );
}
