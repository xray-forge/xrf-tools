import { BaseComponentProps } from "@xrf/ui/element-types";
import { cn } from "@xrf/ui/theme/class-name";
import { ReactElement } from "react";

/**
 * The one vertical rule of a toolbar, between control groups and between the application and the window.
 */
export function EditorToolbarSeparator({
  "data-testid": dataTestId = "editor-toolbar-separator",
  id,
  className,
}: BaseComponentProps): ReactElement {
  return (
    <div
      data-testid={dataTestId}
      aria-hidden={true}
      id={id}
      className={cn("mx-1 h-4.5 w-px shrink-0 bg-divider", className)}
    />
  );
}
