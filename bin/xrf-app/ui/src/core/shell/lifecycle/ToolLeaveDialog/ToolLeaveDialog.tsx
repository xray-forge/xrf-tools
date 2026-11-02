import { useInjection } from "@wirestate/react";
import { formatCount } from "@xrf/format";
import { ConfirmDialog } from "@xrf/ui/dialog/confirm";
import { UnsavedChangesDialog } from "@xrf/ui/dialog/unsaved-changes";
import { ReactElement } from "react";

import { ToolLifecycleService } from "@/core/shell/services/tool-lifecycle";

/** Renders the root lifecycle service's pending leave decision, and an exit asked while work still runs. */
export function ToolLeaveDialog(): ReactElement {
  const lifecycle: ToolLifecycleService = useInjection(ToolLifecycleService);
  const count: number = lifecycle.dirtyCount;
  const unsaved: string = `${formatCount(count)} ${count === 1 ? "file has" : "files have"} edits that are not written to disk.`;

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
