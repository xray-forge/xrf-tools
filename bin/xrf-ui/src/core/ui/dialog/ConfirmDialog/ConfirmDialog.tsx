import { Breakpoint, Button, Dialog, DialogActions, DialogContent, DialogContentText } from "@mui/material";
import { ReactElement, ReactNode, useId } from "react";

import { DialogHeader } from "@/core/ui/dialog/DialogHeader";

interface IConfirmDialogProps {
  isOpen: boolean;
  /** Paints the confirming button as destructive, for commands that overwrite or delete. */
  isDestructive?: boolean;
  /** Holds the confirming button back while the description still asks the user for something. */
  isConfirmDisabled?: boolean;
  title: string;
  /** What the command will do, in terms of what it touches rather than what it is called. */
  description: ReactNode;
  confirmLabel?: string;
  cancelLabel?: string;
  /** Widen it when the description is a summary to read rather than a sentence to acknowledge. */
  maxWidth?: Breakpoint;
  onConfirm: () => void;
  onClose: () => void;
}

/**
 * Confirmation step for commands that cannot be undone.
 *
 * Cancel is focused rather than confirm: the dialog exists because the action is consequential, so the
 * safe option is the one a stray return key should hit.
 */
export function ConfirmDialog({
  isOpen,
  isConfirmDisabled,
  isDestructive,
  title,
  description,
  confirmLabel = "Confirm",
  cancelLabel = "Cancel",
  maxWidth = "xs",
  onConfirm,
  onClose,
}: IConfirmDialogProps): ReactElement {
  const titleId: string = useId();
  const descriptionId: string = useId();

  return (
    <Dialog
      aria-labelledby={titleId}
      aria-describedby={descriptionId}
      open={isOpen}
      maxWidth={maxWidth}
      fullWidth={true}
      onClose={onClose}
    >
      <DialogHeader title={title} titleId={titleId} />

      <DialogContent>
        <DialogContentText id={descriptionId} component={"div"} variant={"body2"}>
          {description}
        </DialogContentText>
      </DialogContent>

      <DialogActions>
        <Button autoFocus={true} size={"small"} onClick={onClose}>
          {cancelLabel}
        </Button>

        <Button
          size={"small"}
          variant={"contained"}
          color={isDestructive ? "error" : "primary"}
          disabled={isConfirmDisabled}
          onClick={onConfirm}
        >
          {confirmLabel}
        </Button>
      </DialogActions>
    </Dialog>
  );
}
