import { BaseComponentProps } from "@xrf/ui/element-types";
import { cn } from "@xrf/ui/theme/class-name";
import { ReactElement, ReactNode } from "react";

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
