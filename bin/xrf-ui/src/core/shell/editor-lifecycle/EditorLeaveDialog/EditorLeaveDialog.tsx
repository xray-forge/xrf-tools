import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { EditorLifecycleService } from "@/core/shell/services/editor-lifecycle";
import { UnsavedChangesDialog } from "@/core/ui/dialog/UnsavedChangesDialog";

/** Renders the root lifecycle service's pending leave decision. */
export function EditorLeaveDialog(): ReactElement {
  const lifecycle: EditorLifecycleService = useInjection(EditorLifecycleService);
  const count: number = lifecycle.dirtyCount;

  return (
    <UnsavedChangesDialog
      isOpen={lifecycle.isLeavePending}
      isSaving={lifecycle.isSaving}
      description={
        `${count} ${count === 1 ? "file has" : "files have"} edits that are not written to disk. ` +
        "Leaving discards them."
      }
      onSave={lifecycle.canSave ? lifecycle.saveAndLeave : null}
      onDiscard={lifecycle.discardAndLeave}
      onClose={lifecycle.stay}
    />
  );
}
