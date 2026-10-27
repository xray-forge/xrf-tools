import { BaseComponentProps } from "@xrf/ui/element-types";
import { Box } from "@xrf/ui/material/box";
import { Typography } from "@xrf/ui/material/typography";
import { ReactElement } from "react";

import { renderHelpText } from "@/core/help/lib/help-text";

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
