import { useInjection } from "@wirestate/react";
import { EditorSideMenu, IEditorSideMenuItem } from "@xrf/ui/editor/side-menu";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { FolderIcon } from "@xrf/ui/icons/folder";
import { SettingsIcon } from "@xrf/ui/icons/settings";
import { TitleIcon } from "@xrf/ui/icons/title";
import { TuneIcon } from "@xrf/ui/icons/tune";
import { ReactElement } from "react";

import { EditorPanelHeader } from "@/core/shell/chrome/EditorPanelHeader";
import { ArchivesPackerService, EPackerSection } from "@/tools/archives-packer/services/archives-packer.service";

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
 * Navigation for the packing configuration, drawn by the shell as the tool's left panel.
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
