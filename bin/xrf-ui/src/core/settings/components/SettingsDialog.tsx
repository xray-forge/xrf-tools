import { default as CloseIcon } from "@mui/icons-material/Close";
import { Box, Button, Dialog, DialogActions, DialogContent, DialogTitle, Divider, IconButton } from "@mui/material";
import { ReactElement } from "react";

import { SettingsBuildInfo } from "@/core/settings/components/SettingsBuildInfo";
import { SettingsForm } from "@/core/settings/components/SettingsForm";

export interface ISettingsDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

/**
 * Settings as a proper desktop dialog: titled, explicitly dismissable, and wide enough to read a path.
 */
export function SettingsDialog({ isOpen, onClose }: ISettingsDialogProps): ReactElement {
  return (
    <Dialog fullWidth maxWidth={"sm"} open={isOpen} onClose={onClose}>
      <DialogTitle sx={{ display: "flex", alignItems: "center", paddingY: 1.5, paddingRight: 1 }}>
        Settings
        <Box sx={{ flexGrow: 1 }} />
        <IconButton onClick={onClose}>
          <CloseIcon fontSize={"small"} />
        </IconButton>
      </DialogTitle>

      <Divider />

      <DialogContent sx={{ paddingY: 3 }}>
        <SettingsForm />

        <Divider sx={{ marginY: 3 }} />

        <SettingsBuildInfo />
      </DialogContent>

      <Divider />

      <DialogActions sx={{ paddingX: 3, paddingY: 1.5 }}>
        <Button variant={"contained"} onClick={onClose}>
          Done
        </Button>
      </DialogActions>
    </Dialog>
  );
}
