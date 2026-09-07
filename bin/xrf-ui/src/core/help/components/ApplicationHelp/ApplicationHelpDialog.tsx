import { Dialog, DialogContent } from "@mui/material";
import { ReactElement, useId } from "react";

import { ApplicationHelpContent } from "@/core/help/components/ApplicationHelpContent";
import { IApplicationDescriptor, IApplicationHelp } from "@/core/routing/application";
import { DialogHeader } from "@/core/ui/dialog/DialogHeader";
import { StyledComponentProps } from "@/lib/dom/element-types";

export interface IApplicationHelpDialogProps extends StyledComponentProps {
  application: IApplicationDescriptor;
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
  sx,
  application,
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
      sx={sx}
      open={isOpen}
      maxWidth={"md"}
      fullWidth={true}
      onClose={onClose}
    >
      <DialogHeader
        title={application.label}
        titleId={titleId}
        icon={application.icon}
        closeLabel={"Close help"}
        onClose={onClose}
      />

      <DialogContent>
        <ApplicationHelpContent help={help} onNavigated={onClose} />
      </DialogContent>
    </Dialog>
  );
}
