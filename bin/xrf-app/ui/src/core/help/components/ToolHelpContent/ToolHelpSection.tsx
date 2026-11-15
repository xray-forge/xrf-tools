import { renderCodeSpans } from "@xrf/ui/code/spans";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { Box } from "@xrf/ui/layout/stack";
import { Typography } from "@xrf/ui/typography";
import { ReactElement } from "react";

export interface IToolHelpSectionProps extends BaseComponentProps {
  title: string;
  items: ReadonlyArray<string>;
  isOrdered?: boolean;
}

/**
 * One rubric section: its heading over a tight list.
 */
export function ToolHelpSection({
  "data-testid": dataTestId = "tool-help-section",
  id,
  className,
  title,
  items,
  isOrdered,
}: IToolHelpSectionProps): ReactElement {
  return (
    <DetailSection data-testid={dataTestId} id={id} className={className} title={title}>
      <Box className={"m-0 pl-5"} component={isOrdered ? "ol" : "ul"}>
        {items.map((item: string, index: number) => (
          <Typography key={index} className={"mb-1 leading-panel"} component={"li"} variant={"body2"}>
            {renderCodeSpans(item)}
          </Typography>
        ))}
      </Box>
    </DetailSection>
  );
}
