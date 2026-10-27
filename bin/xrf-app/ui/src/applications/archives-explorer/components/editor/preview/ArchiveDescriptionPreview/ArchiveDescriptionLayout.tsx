import { BaseComponentProps } from "@xrf/ui/element-types";
import { cn } from "@xrf/ui/theme/class-name";
import { ReactElement, ReactNode } from "react";

interface IArchiveDescriptionLayoutProps extends BaseComponentProps {
  children: ReactNode;
}

/**
 * The scrolling pane every format description is read in.
 */
export function ArchiveDescriptionLayout({
  "data-testid": dataTestId,
  id,
  className,
  children,
}: IArchiveDescriptionLayoutProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("min-h-0 min-w-0 grow overflow-y-auto", className)}>
      <div className={"mx-auto max-w-reading"}>{children}</div>
    </div>
  );
}
