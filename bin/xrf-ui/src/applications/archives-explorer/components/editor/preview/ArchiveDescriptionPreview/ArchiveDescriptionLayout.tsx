import { ReactElement, ReactNode } from "react";

import { BaseComponentProps } from "@/lib/dom/element-types";

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
    <div data-testid={dataTestId} id={id} className={`min-h-0 min-w-0 grow overflow-y-auto ${className ?? ""}`}>
      <div className={"mx-auto max-w-reading"}>{children}</div>
    </div>
  );
}
