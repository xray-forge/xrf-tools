import { Fragment, ReactElement } from "react";

import { IEditorStatusDetail } from "@/core/shell/editor-shell";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IApplicationStatusDetailsProps extends BaseComponentProps {
  details: ReadonlyArray<IEditorStatusDetail>;
}

/**
 * A status segment's hover: its figures as label and value columns.
 */
export function ApplicationStatusDetails({
  "data-testid": dataTestId = "application-status-details",
  id,
  className,
  details,
}: IApplicationStatusDetailsProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("grid grid-cols-[auto_auto] gap-x-4", className)}>
      {details.map((detail: IEditorStatusDetail, index: number) => (
        <Fragment key={index}>
          <div className={cn(detail.isNested ? "pl-3 opacity-70" : null)}>{detail.label}</div>
          <div className={cn("text-right tabular-nums", detail.isNested ? "opacity-70" : null)}>{detail.value}</div>
        </Fragment>
      ))}
    </div>
  );
}
