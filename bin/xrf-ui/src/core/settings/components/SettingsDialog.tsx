import { default as CloseIcon } from "@mui/icons-material/Close";
import {
  Box,
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  Divider,
  IconButton,
  List,
  ListItemButton,
  ListItemText,
} from "@mui/material";
import { ReactElement, useState } from "react";

import { SettingsBuildInfo } from "@/core/settings/components/SettingsBuildInfo";
import { SettingsGeneralSection } from "@/core/settings/components/SettingsGeneralSection";
import { SettingsPathsSection } from "@/core/settings/components/SettingsPathsSection";
import { inline } from "@/lib/callbacks/inline";

/** The sections settings are grouped into, in the order the rail lists them. */
const enum ESettingsSection {
  GENERAL = "general",
  PATHS = "paths",
  ABOUT = "about",
}

const SECTION_LABELS: Record<ESettingsSection, string> = {
  [ESettingsSection.GENERAL]: "General",
  [ESettingsSection.PATHS]: "Paths",
  [ESettingsSection.ABOUT]: "About",
};

const SECTIONS: ReadonlyArray<ESettingsSection> = [
  ESettingsSection.GENERAL,
  ESettingsSection.PATHS,
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
  const [section, setSection] = useState<ESettingsSection>(ESettingsSection.GENERAL);

  return (
    <Dialog fullWidth maxWidth={"md"} open={isOpen} onClose={onClose}>
      <DialogTitle sx={{ display: "flex", alignItems: "center", paddingY: 1.5, paddingRight: 1 }}>
        Settings
        <Box sx={{ flexGrow: 1 }} />
        <IconButton onClick={onClose}>
          <CloseIcon fontSize={"small"} />
        </IconButton>
      </DialogTitle>

      <Divider />

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
            paddingX: 3,
            paddingY: 3,
            backgroundColor: "background.default",
          }}
        >
          {inline(() => {
            switch (section) {
              case ESettingsSection.GENERAL:
                return <SettingsGeneralSection />;
              case ESettingsSection.PATHS:
                return <SettingsPathsSection />;
              case ESettingsSection.ABOUT:
                return <SettingsBuildInfo />;
            }
          })}
        </Box>
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
