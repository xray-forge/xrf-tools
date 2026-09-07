import { Button, CircularProgress, Dialog, DialogActions, DialogContent, DialogContentText } from "@mui/material";
import { ReactElement, ReactNode, useId } from "react";

import { DialogHeader } from "@/core/ui/dialog/DialogHeader";
import { Nullable } from "@/lib/types/general";

interface IUnsavedChangesDialogProps {
  isOpen: boolean;
  /** Whether a save asked for from here is still running, which holds every button until it answers. */
  isSaving?: boolean;
  title?: string;
  /** What would be lost, said in terms of what is holding it rather than what the buttons are called. */
  description: ReactNode;
  /** Writes the pending work and then leaves. */
  onSave: Nullable<() => void>;
  onDiscard: () => void;
  onClose: () => void;
}

/**
 * The question asked when leaving would abandon work that is not on disk.
 */
export function UnsavedChangesDialog({
  isOpen,
  isSaving = false,
  title = "Leave without saving?",
  description,
  onSave,
  onDiscard,
  onClose,
}: IUnsavedChangesDialogProps): ReactElement {
  const titleId: string = useId();
  const descriptionId: string = useId();

  return (
    <Dialog
      aria-labelledby={titleId}
      aria-describedby={descriptionId}
      open={isOpen}
      maxWidth={"xs"}
      fullWidth={true}
      onClose={isSaving ? undefined : onClose}
    >
      <DialogHeader title={title} titleId={titleId} />

      <DialogContent>
        <DialogContentText id={descriptionId} component={"div"} variant={"body2"}>
          {description}
        </DialogContentText>
      </DialogContent>

      <DialogActions>
        <Button autoFocus={true} size={"small"} disabled={isSaving} onClick={onClose}>
          Stay
        </Button>

        <Button size={"small"} color={"error"} disabled={isSaving} onClick={onDiscard}>
          Discard and leave
        </Button>

        {onSave ? (
          <Button
            size={"small"}
            variant={"contained"}
            disabled={isSaving}
            startIcon={isSaving ? <CircularProgress size={14} color={"inherit"} /> : null}
            onClick={onSave}
          >
            {isSaving ? "Saving" : "Save and leave"}
          </Button>
        ) : null}
      </DialogActions>
    </Dialog>
  );
}
