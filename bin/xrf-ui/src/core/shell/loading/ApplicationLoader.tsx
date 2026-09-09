import { ReactElement } from "react";

import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { getApplicationBackgroundSx } from "@/core/theme/application-background";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { BaseComponentProps } from "@/lib/dom/element-types";

/**
 * Shown while an application is still arriving - its chunk fetched, or its services provisioning.
 */
export function ApplicationLoader({ "data-testid": dataTestId, id, className }: BaseComponentProps): ReactElement {
  return (
    <EditorLayout toolbar={<EditorToolbar />} sx={getApplicationBackgroundSx}>
      <DelayedProgress data-testid={dataTestId} id={id} className={className} />
    </EditorLayout>
  );
}
