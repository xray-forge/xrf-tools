import { DialogHeader } from "@xrf/ui/dialog/header";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Dialog, DialogContent } from "@xrf/ui/material/dialog";
import { ReactElement, useId } from "react";

import { ToolHelpContent } from "@/core/help/components/ToolHelpContent";
import { IToolHelp } from "@/core/routing/tool";

export interface IToolHelpDialogProps extends BaseComponentProps {
  /** What the help is about, named as its own screen names itself. */
  title: string;
  icon: ReactElement;
  help: IToolHelp;
  isOpen: boolean;
  onClose: () => void;
}

/**
 * Help as a modal over the working tool.
 */
export function ToolHelpDialog({
  "data-testid": dataTestId = "tool-help-dialog",
  id,
  className,
  title,
  icon,
  help,
  isOpen,
  onClose,
}: IToolHelpDialogProps): ReactElement {
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
        <ToolHelpContent help={help} onNavigated={onClose} />
      </DialogContent>
    </Dialog>
  );
}
