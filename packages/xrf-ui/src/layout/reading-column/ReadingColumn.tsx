import { ReactElement, ReactNode } from "react";

import { BaseComponentProps } from "#/element-types";
import { cn } from "#/theme";

interface IReadingColumnProps extends BaseComponentProps {
  children: ReactNode;
}

/**
 * A pane that scrolls a centred column at the reading width, anchored to the top because what it holds is read down.
 */
export function ReadingColumn({
  "data-testid": dataTestId,
  id,
  className,
  children,
}: IReadingColumnProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("min-h-0 min-w-0 grow overflow-y-auto", className)}>
      <div className={"mx-auto max-w-reading"}>{children}</div>
    </div>
  );
}
