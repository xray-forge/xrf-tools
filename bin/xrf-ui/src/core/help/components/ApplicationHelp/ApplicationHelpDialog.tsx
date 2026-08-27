import { default as CloseIcon } from "@mui/icons-material/Close";
import { Box, Dialog, DialogContent, DialogTitle, IconButton, Tooltip } from "@mui/material";
import { ReactElement } from "react";

import { ApplicationHelpContent } from "@/core/help/components/ApplicationHelpContent";
import { IApplicationDescriptor, IApplicationHelp } from "@/core/routing/application";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IApplicationHelpDialogProps extends BaseComponentProps {
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
  return (
    <Dialog
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={sx}
      open={isOpen}
      maxWidth={"md"}
      fullWidth={true}
      onClose={onClose}
    >
      <DialogTitle sx={{ display: "flex", alignItems: "center", gap: 1, paddingY: 1.5 }}>
        {application.icon}

        <Box component={"span"} sx={{ flexGrow: 1, minWidth: 0 }}>
          {application.label}
        </Box>

        <Tooltip title={"Close"}>
          <IconButton aria-label={"Close help"} size={"small"} onClick={onClose}>
            <CloseIcon fontSize={"inherit"} />
          </IconButton>
        </Tooltip>
      </DialogTitle>

      <DialogContent dividers={true}>
        <ApplicationHelpContent help={help} onNavigated={onClose} />
      </DialogContent>
    </Dialog>
  );
}
