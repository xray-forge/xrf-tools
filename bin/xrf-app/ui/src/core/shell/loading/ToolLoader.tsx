import { EditorLayout } from "@xrf/ui/editor/layout";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { DelayedProgress } from "@xrf/ui/layout/delayed-progress";
import { ReactElement } from "react";

import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";

/**
 * Shown while a tool is still arriving - its chunk fetched, or its services provisioning.
 */
export function ToolLoader({ "data-testid": dataTestId, id, className }: BaseComponentProps): ReactElement {
  return (
    <EditorLayout toolbar={<EditorToolbar />}>
      <DelayedProgress data-testid={dataTestId} id={id} className={className} />
    </EditorLayout>
  );
}
