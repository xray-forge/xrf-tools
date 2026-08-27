import { default as FolderIcon } from "@mui/icons-material/Folder";
import { default as SettingsIcon } from "@mui/icons-material/Settings";
import { default as TitleIcon } from "@mui/icons-material/Title";
import { default as TuneIcon } from "@mui/icons-material/Tune";
import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { EPackerSection, PackerService } from "@/applications/archives-packer/services/packer";
import { EditorPanelHeader } from "@/core/shell/editor/EditorPanelHeader";
import { EditorSideMenu, IEditorSideMenuItem } from "@/core/shell/editor/EditorSideMenu";

/** Named once, because the panel and its own heading must not drift apart. */
export const PACKER_SECTIONS_PANEL_LABEL: string = "Configuration";

const SECTIONS: Array<{ id: EPackerSection; label: string; description: string; icon: ReactElement }> = [
  {
    id: EPackerSection.OUTPUT,
    label: "Source and output",
    description: "Where it comes from and lands",
    icon: <FolderIcon />,
  },
  { id: EPackerSection.SELECTION, label: "Selection", description: "What goes in", icon: <TuneIcon /> },
  { id: EPackerSection.HEADER, label: "Header", description: "Where it mounts", icon: <TitleIcon /> },
  { id: EPackerSection.OPTIONS, label: "Options", description: "How it is written", icon: <SettingsIcon /> },
];

/**
 * Navigation for the packing configuration, drawn by the shell as the application's left panel.
 */
export function PackerSectionsMenu(): ReactElement {
  const packerService: PackerService = useInjection(PackerService);

  const items: Array<IEditorSideMenuItem> = SECTIONS.map((section) => ({
    label: section.label,
    description: section.description,
    icon: section.icon,
    isSelected: packerService.section === section.id,
    onClick: () => packerService.setSection(section.id),
  }));

  return <EditorSideMenu header={<EditorPanelHeader title={PACKER_SECTIONS_PANEL_LABEL} />} sections={items} />;
}
