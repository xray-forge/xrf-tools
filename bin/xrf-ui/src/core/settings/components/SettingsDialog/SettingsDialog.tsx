import { Box, Button, Dialog, DialogActions, DialogContent, List, ListItemButton, ListItemText } from "@mui/material";
import { ReactElement, useId, useState } from "react";

import { mergeSx } from "@/core/theme/merge-sx";
import { getWellFillSx } from "@/core/theme/surface";
import { DIALOG } from "@/core/theme/tokens";
import { DialogHeader } from "@/core/ui/dialog/DialogHeader";
import { inline } from "@/lib/callbacks/inline";

import { SettingsAboutSection } from "./SettingsAboutSection";
import { SettingsGeneralSection } from "./SettingsGeneralSection";
import { SettingsIpcSection } from "./SettingsIpcSection";
import { SettingsJobsSection } from "./SettingsJobsSection";
import { SettingsStorageSection } from "./SettingsStorageSection";

/** The sections settings are grouped into, in the order the rail lists them. */
enum EDetailSection {
  GENERAL = "general",
  STORAGE = "storage",
  IPC = "ipc",
  JOBS = "jobs",
  ABOUT = "about",
}

const SECTION_LABELS: Record<EDetailSection, string> = {
  [EDetailSection.GENERAL]: "General",
  [EDetailSection.STORAGE]: "Storage",
  [EDetailSection.IPC]: "IPC",
  [EDetailSection.JOBS]: "Jobs",
  [EDetailSection.ABOUT]: "About",
};

const SECTIONS: ReadonlyArray<EDetailSection> = [
  EDetailSection.GENERAL,
  EDetailSection.STORAGE,
  EDetailSection.IPC,
  EDetailSection.JOBS,
  EDetailSection.ABOUT,
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
  const [section, setSection] = useState<EDetailSection>(EDetailSection.GENERAL);

  return (
    <Dialog aria-labelledby={titleId} fullWidth maxWidth={"md"} open={isOpen} onClose={onClose}>
      <DialogHeader title={"Settings"} titleId={titleId} closeLabel={"Close settings"} onClose={onClose} />

      <DialogContent sx={{ display: "flex", padding: 0, height: 420, maxHeight: "60vh" }}>
        <List
          dense
          disablePadding
          sx={mergeSx(getWellFillSx, {
            width: 148,
            flexShrink: 0,
            paddingY: 1,
            borderRight: 1,
            borderColor: "divider",
          })}
        >
          {SECTIONS.map((it: EDetailSection) => (
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
          }}
        >
          {inline(() => {
            switch (section) {
              case EDetailSection.GENERAL:
                return <SettingsGeneralSection />;
              case EDetailSection.STORAGE:
                return <SettingsStorageSection />;
              case EDetailSection.IPC:
                return <SettingsIpcSection />;
              case EDetailSection.JOBS:
                return <SettingsJobsSection />;
              case EDetailSection.ABOUT:
                return <SettingsAboutSection />;
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
