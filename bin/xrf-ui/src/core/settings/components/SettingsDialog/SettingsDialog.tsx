import { Box, Button, Dialog, DialogActions, DialogContent, List, ListItemButton, ListItemText } from "@mui/material";
import { ReactElement, useId, useState } from "react";

import { SettingsBuildInfo } from "@/core/settings/components/SettingsBuildInfo";
import { SettingsGeneralSection } from "@/core/settings/components/SettingsGeneralSection";
import { SettingsStorageSection } from "@/core/settings/components/SettingsStorageSection";
import { DIALOG } from "@/core/theme/tokens";
import { DialogHeader } from "@/core/ui/dialog/DialogHeader";
import { inline } from "@/lib/callbacks/inline";

/** The sections settings are grouped into, in the order the rail lists them. */
const enum ESettingsSection {
  GENERAL = "general",
  STORAGE = "storage",
  ABOUT = "about",
}

const SECTION_LABELS: Record<ESettingsSection, string> = {
  [ESettingsSection.GENERAL]: "General",
  [ESettingsSection.STORAGE]: "Storage",
  [ESettingsSection.ABOUT]: "About",
};

const SECTIONS: ReadonlyArray<ESettingsSection> = [
  ESettingsSection.GENERAL,
  ESettingsSection.STORAGE,
  ESettingsSection.ABOUT,
];

export interface ISettingsDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

/**
 * Settings as a proper desktop dialog: titled, explicitly dismissable, and wide enough to read a path.
 */
export function SettingsDialog({ isOpen, onClose }: ISettingsDialogProps): ReactElement {
  const titleId: string = useId();
  const [section, setSection] = useState<ESettingsSection>(ESettingsSection.GENERAL);

  return (
    <Dialog aria-labelledby={titleId} fullWidth maxWidth={"md"} open={isOpen} onClose={onClose}>
      <DialogHeader title={"Settings"} titleId={titleId} closeLabel={"Close settings"} onClose={onClose} />

      <DialogContent sx={{ display: "flex", padding: 0, height: 420, maxHeight: "60vh" }}>
        <List
          dense
          disablePadding
          sx={{
            width: 148,
            flexShrink: 0,
            paddingY: 1,
            borderRight: 1,
            borderColor: "divider",
            backgroundColor: "background.default",
          }}
        >
          {SECTIONS.map((it: ESettingsSection) => (
            <ListItemButton key={it} selected={section === it} onClick={() => setSection(it)}>
              <ListItemText primary={SECTION_LABELS[it]} />
            </ListItemButton>
          ))}
        </List>

        <Box
          sx={{
            flexGrow: 1,
            minWidth: 0,
            overflowY: "auto",
            paddingX: DIALOG.paddingX,
            paddingY: DIALOG.contentPaddingY,
            backgroundColor: "background.default",
          }}
        >
          {inline(() => {
            switch (section) {
              case ESettingsSection.GENERAL:
                return <SettingsGeneralSection />;
              case ESettingsSection.STORAGE:
                return <SettingsStorageSection />;
              case ESettingsSection.ABOUT:
                return <SettingsBuildInfo />;
            }
          })}
        </Box>
      </DialogContent>

      <DialogActions>
        <Button size={"small"} variant={"contained"} onClick={onClose}>
          Done
        </Button>
      </DialogActions>
    </Dialog>
  );
}
