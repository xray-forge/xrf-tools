import { Dialog, DialogContent } from "@mui/material";
import { ReactElement, useId } from "react";

import { ApplicationHelpContent } from "@/core/help/components/ApplicationHelpContent";
import { IApplicationHelp } from "@/core/routing/application";
import { DialogHeader } from "@/core/ui/dialog/DialogHeader";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IApplicationHelpDialogProps extends BaseComponentProps {
  /** What the help is about, named as its own screen names itself. */
  title: string;
  icon: ReactElement;
  help: IApplicationHelp;
  isOpen: boolean;
  onClose: () => void;
}

/**
 * Help as a modal over the working tool.
 */
export function ApplicationHelpDialog({
  "data-testid": dataTestId = "application-help-dialog",
  id,
  className,
  title,
  icon,
  help,
  isOpen,
  onClose,
}: IApplicationHelpDialogProps): ReactElement {
  const titleId: string = useId();

  return (
    <Dialog
      data-testid={dataTestId}
      aria-labelledby={titleId}
      id={id}
      className={className}
      open={isOpen}
      maxWidth={"md"}
      fullWidth={true}
      onClose={onClose}
    >
      <DialogHeader title={title} titleId={titleId} icon={icon} closeLabel={"Close help"} onClose={onClose} />

      <DialogContent>
        <ApplicationHelpContent help={help} onNavigated={onClose} />
      </DialogContent>
    </Dialog>
  );
}
