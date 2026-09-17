import { Box, Typography } from "@mui/material";
import { ReactElement } from "react";

import { renderHelpText } from "@/core/help/lib/help-text";
import { StyledComponentProps } from "@/lib/dom/element-types";

export interface IApplicationHelpSectionProps extends StyledComponentProps {
  title: string;
  items: ReadonlyArray<string>;
  isOrdered?: boolean;
}

/**
 * One rubric section: a stated-color heading over a tight list.
 */
export function ApplicationHelpSection({
  "data-testid": dataTestId = "application-help-section",
  id,
  className,
  sx,
  title,
  items,
  isOrdered,
}: IApplicationHelpSectionProps): ReactElement {
  return (
    <Box data-testid={dataTestId} id={id} className={className} sx={sx}>
      <Typography className={"mb-1 text-text-primary"} variant={"subtitle2"}>
        {title}
      </Typography>

      <Box className={"m-0 pl-5"} component={isOrdered ? "ol" : "ul"}>
        {items.map((item: string, index: number) => (
          <Typography key={index} className={"mb-1"} component={"li"} variant={"body2"} sx={{ lineHeight: 1.55 }}>
            {renderHelpText(item)}
          </Typography>
        ))}
      </Box>
    </Box>
  );
}
