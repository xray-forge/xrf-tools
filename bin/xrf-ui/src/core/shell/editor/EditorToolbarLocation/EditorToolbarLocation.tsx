import { Tooltip } from "@mui/material";
import { ReactElement } from "react";

import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

/**
 * Where an open session actually is on disk.
 */
export interface IEditorLocation {
  /** The directory, installation root or archive the session was opened on. */
  path: string;
}

interface IEditorToolbarLocationProps extends BaseComponentProps {
  location: IEditorLocation;
}

/**
 * Where the open session is, on the toolbar's last crumb.
 *
 * Truncated from the left by the toolbar, which is what keeps the end of a path - the part that identifies the place -
 * on screen when the beginning does not fit. The tooltip carries it in full, since the whole reason to show a location
 * is the times somebody needs to copy or check it.
 */
export function EditorToolbarLocation({
  "data-testid": dataTestId = "editor-toolbar-location",
  id,
  className,
  location,
}: IEditorToolbarLocationProps): ReactElement {
  return (
    <Tooltip title={location.path}>
      <span data-testid={dataTestId} id={id} className={cn("font-monospace", className)}>
        {location.path}
      </span>
    </Tooltip>
  );
}
