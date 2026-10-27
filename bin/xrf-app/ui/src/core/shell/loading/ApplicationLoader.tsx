import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";

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
