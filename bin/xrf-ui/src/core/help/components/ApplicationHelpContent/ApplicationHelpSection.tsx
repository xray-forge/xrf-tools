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
  sx,
  title,
  items,
  isOrdered,
}: IApplicationHelpSectionProps): ReactElement {
  return (
    <Box data-testid={dataTestId} id={id} className={className} sx={sx}>
      <Typography variant={"subtitle2"} sx={{ color: "text.primary", marginBottom: 0.5 }}>
        {title}
      </Typography>

      <Box component={isOrdered ? "ol" : "ul"} sx={{ margin: 0, paddingLeft: 2.5 }}>
        {items.map((item: string, index: number) => (
          <Typography key={index} component={"li"} variant={"body2"} sx={{ marginBottom: 0.5, lineHeight: 1.55 }}>
            {renderHelpText(item)}
          </Typography>
        ))}
      </Box>
    </Box>
  );
}
