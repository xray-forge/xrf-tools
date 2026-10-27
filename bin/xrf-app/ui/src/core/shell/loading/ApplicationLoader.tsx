import { BaseComponentProps } from "@xrf/ui/element-types";
import { DelayedProgress } from "@xrf/ui/layout/DelayedProgress";
import { EditorLayout } from "@xrf/ui/shell/editor/EditorLayout";
import { ReactElement } from "react";

import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";

/**
 * Shown while an application is still arriving - its chunk fetched, or its services provisioning.
 */
export function ApplicationLoader({ "data-testid": dataTestId, id, className }: BaseComponentProps): ReactElement {
  return (
    <EditorLayout toolbar={<EditorToolbar />}>
      <DelayedProgress data-testid={dataTestId} id={id} className={className} />
    </EditorLayout>
  );
}
