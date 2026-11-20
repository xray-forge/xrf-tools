import { ConfirmDialog } from "@xrf/ui/dialog/confirm";
import { ReactElement } from "react";

import { IReplaceRetry } from "@/core/jobs/lib/use-replace-retry";

interface IReplaceConfirmDialogProps {
  /** The run whose refusal it asks about; open while one is held. */
  retry: Pick<IReplaceRetry<unknown>, "refusal" | "replace" | "dismiss">;
}

/**
 * Asks whether a run refused over files already there may replace them, naming them as the refusal did; replacing
 * runs it again, forced.
 */
export function ReplaceConfirmDialog({ retry }: IReplaceConfirmDialogProps): ReactElement {
  return (
    <ConfirmDialog
      isDestructive
      isOpen={retry.refusal !== null}
      maxWidth={"sm"}
      title={"Replace existing files?"}
      description={retry.refusal}
      confirmLabel={"Replace"}
      onClose={retry.dismiss}
      onConfirm={() => void retry.replace()}
    />
  );
}
