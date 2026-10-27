import { useInjection } from "@wirestate/react";
import { ConfirmDialog } from "@xrf/ui/dialog/ConfirmDialog";
import { UnsavedChangesDialog } from "@xrf/ui/dialog/UnsavedChangesDialog";
import { ReactElement } from "react";

import { EditorLifecycleService } from "@/core/shell/services/editor-lifecycle";

/** Renders the root lifecycle service's pending leave decision, and an exit asked while work still runs. */
export function EditorLeaveDialog(): ReactElement {
  const lifecycle: EditorLifecycleService = useInjection(EditorLifecycleService);
  const count: number = lifecycle.dirtyCount;
  const unsaved: string = `${count} ${count === 1 ? "file has" : "files have"} edits that are not written to disk.`;

  return (
    <>
      <UnsavedChangesDialog
        isOpen={lifecycle.isLeavePending}
        isSaving={lifecycle.isSaving}
        description={`${unsaved} Leaving discards them.`}
        onSave={lifecycle.canSave ? lifecycle.saveAndLeave : null}
        onDiscard={lifecycle.discardAndLeave}
        onClose={lifecycle.stay}
      />

      <ConfirmDialog
        isOpen={lifecycle.isExitPending}
        isDestructive={true}
        title={"Exit while working?"}
        description={
          "An operation is still running, and exiting stops it before it finishes." +
          (count > 0 ? ` ${unsaved} Exiting discards them.` : "")
        }
        confirmLabel={"Exit anyway"}
        cancelLabel={"Stay"}
        onConfirm={lifecycle.exitAnyway}
        onClose={lifecycle.stay}
      />
    </>
  );
}
