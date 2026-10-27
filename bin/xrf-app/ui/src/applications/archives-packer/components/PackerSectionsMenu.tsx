import { useInjection } from "@wirestate/react";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { FolderIcon, SettingsIcon, TitleIcon, TuneIcon } from "@xrf/ui/icons";
import { ReactElement } from "react";

import { ArchivesPackerService, EPackerSection } from "@/applications/archives-packer/services/archives-packer.service";
import { EditorPanelHeader } from "@/core/shell/editor/EditorPanelHeader";
import { EditorSideMenu, IEditorSideMenuItem } from "@/core/shell/editor/EditorSideMenu";

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
export function PackerSectionsMenu({
  "data-testid": dataTestId = "packer-sections-menu",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const packerService: ArchivesPackerService = useInjection(ArchivesPackerService);

  const items: Array<IEditorSideMenuItem> = SECTIONS.map((section) => ({
    label: section.label,
    description: section.description,
    icon: section.icon,
    isSelected: packerService.section === section.id,
    onClick: () => packerService.setSection(section.id),
  }));

  return (
    <EditorSideMenu
      data-testid={dataTestId}
      id={id}
      className={className}
      header={<EditorPanelHeader title={"Configuration"} />}
      sections={items}
    />
  );
}
