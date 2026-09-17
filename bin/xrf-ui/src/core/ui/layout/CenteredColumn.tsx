import { ReactElement, ReactNode } from "react";

import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ICenteredColumnProps extends BaseComponentProps {
  children: ReactNode;
}

/**
 * Full-size flex column that centers its children both axes.
 */
export function CenteredColumn({
  "data-testid": dataTestId,
  id,
  className,
  children,
}: ICenteredColumnProps): ReactElement {
  return (
    <div
      data-testid={dataTestId}
      id={id}
      className={cn("flex h-full w-full flex-col items-center justify-center gap-2", className)}
    >
      {children}
    </div>
  );
}
