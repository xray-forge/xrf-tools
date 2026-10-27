import { useInjection } from "@wirestate/react";
import { DialogHeader } from "@xrf/ui/dialog/DialogHeader";
import {
  Box,
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  List,
  ListItemButton,
  ListItemText,
} from "@xrf/ui/material";
import { FORM_SURFACE_SX } from "@xrf/ui/theme/form-surface";
import { ReactElement, useId } from "react";

import { ESettingsSection, SETTINGS_SECTION_LABELS, SETTINGS_SECTIONS } from "@/core/settings/lib/settings-section";
import { SettingsDialogService } from "@/core/settings/services/settings-dialog";
import { inline } from "@/lib/callbacks/inline";

import { SettingsAboutSection } from "./SettingsAboutSection";
import { SettingsGeneralSection } from "./SettingsGeneralSection";
import { SettingsIpcSection } from "./SettingsIpcSection";
import { SettingsJobsSection } from "./SettingsJobsSection";
import { SettingsRenderSection } from "./SettingsRenderSection";
import { SettingsStorageSection } from "./SettingsStorageSection";

/**
 * Settings as a proper desktop dialog: titled, explicitly dismissable, and wide enough to read a path.
 */
export function SettingsDialog(): ReactElement {
  const settingsDialogService: SettingsDialogService = useInjection(SettingsDialogService);
  const titleId: string = useId();

  const { isOpen, section, close: onClose, setSection } = settingsDialogService;

  return (
    <Dialog aria-labelledby={titleId} fullWidth maxWidth={"md"} open={isOpen} onClose={onClose}>
      <DialogHeader title={"Settings"} titleId={titleId} closeLabel={"Close settings"} onClose={onClose} />

      <DialogContent className={"flex h-105 max-h-dialog-height p-0"}>
        <List className={"w-37 shrink-0 border-r border-divider bg-well"} dense={true} disablePadding={true}>
          {SETTINGS_SECTIONS.map((it: ESettingsSection) => (
            <ListItemButton key={it} selected={section === it} onClick={() => setSection(it)}>
              <ListItemText primary={SETTINGS_SECTION_LABELS[it]} />
            </ListItemButton>
          ))}
        </List>

        <Box className={"min-w-0 grow overflow-y-auto px-dialog py-6"} sx={FORM_SURFACE_SX}>
          {inline(() => {
            switch (section) {
              case ESettingsSection.GENERAL:
                return <SettingsGeneralSection />;
              case ESettingsSection.RENDER:
                return <SettingsRenderSection />;
              case ESettingsSection.STORAGE:
                return <SettingsStorageSection />;
              case ESettingsSection.IPC:
                return <SettingsIpcSection />;
              case ESettingsSection.JOBS:
                return <SettingsJobsSection />;
              case ESettingsSection.ABOUT:
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
