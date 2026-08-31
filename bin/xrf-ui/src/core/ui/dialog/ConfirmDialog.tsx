import {
  Breakpoint,
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogContentText,
  DialogTitle,
} from "@mui/material";
import { ReactElement, ReactNode } from "react";

export interface IConfirmDialogProps {
  isOpen: boolean;
  /** Paints the confirming button as destructive, for commands that overwrite or delete. */
  isDestructive?: boolean;
  /** Holds the confirming button back while the description still asks the user for something. */
  isConfirmDisabled?: boolean;
  /** Widen it when the description is a summary to read rather than a sentence to acknowledge. */
  title: string;
  /** What the command will do, in terms of what it touches rather than what it is called. */
  description: ReactNode;
  confirmLabel?: string;
  cancelLabel?: string;
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
  return (
    <Dialog open={isOpen} maxWidth={maxWidth} fullWidth={true} onClose={onClose}>
      <DialogTitle sx={{ paddingBottom: 1 }}>{title}</DialogTitle>

      <DialogContent>
        <DialogContentText component={"div"} variant={"body2"}>
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
