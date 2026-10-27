import { ReactElement } from "react";

import { BaseComponentProps } from "#/element-types";
import { cn } from "#/theme/class-name";

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
