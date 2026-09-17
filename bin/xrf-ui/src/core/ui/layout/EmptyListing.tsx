import { Typography } from "@mui/material";
import { ReactElement } from "react";

import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IEmptyListingProps extends BaseComponentProps {
  /** What is absent, and why, in one line. */
  label: string;
}

/**
 * Stands in for the rows a searchable listing would hold, centred in the column they would have filled.
 */
export function EmptyListing({
  "data-testid": dataTestId = "empty-listing",
  id,
  className,
  label,
}: IEmptyListingProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("p-4 text-center", className)}>
      <Typography className={"text-text-secondary"} variant={"body2"}>
        {label}
      </Typography>
    </div>
  );
}
