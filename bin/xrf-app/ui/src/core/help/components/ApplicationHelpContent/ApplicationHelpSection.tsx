import { Box, Typography } from "@mui/material";
import { ReactElement } from "react";

import { renderHelpText } from "@/core/help/lib/help-text";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IApplicationHelpSectionProps extends BaseComponentProps {
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
  title,
  items,
  isOrdered,
}: IApplicationHelpSectionProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={className}>
      <Typography className={"mb-1 text-text-primary"} variant={"subtitle2"}>
        {title}
      </Typography>

      <Box className={"m-0 pl-5"} component={isOrdered ? "ol" : "ul"}>
        {items.map((item: string, index: number) => (
          <Typography key={index} className={"mb-1 leading-panel"} component={"li"} variant={"body2"}>
            {renderHelpText(item)}
          </Typography>
        ))}
      </Box>
    </div>
  );
}
