import { useInjection } from "@wirestate/react";
import { EditorSideMenu, IEditorSideMenuItem } from "@xrf/ui/editor/side-menu";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { CompareArrowsIcon } from "@xrf/ui/icons/compare-arrows";
import { FolderIcon } from "@xrf/ui/icons/folder";
import { SettingsIcon } from "@xrf/ui/icons/settings";
import { TitleIcon } from "@xrf/ui/icons/title";
import { TuneIcon } from "@xrf/ui/icons/tune";
import { ReactElement } from "react";

import { EditorPanelHeader } from "@/core/shell/editor/EditorPanelHeader";
import { ArchivesPatcherService, EPatcherSection } from "@/tools/archives-patcher/services/archives-patcher.service";

/** Named once, because the panel and its own heading must not drift apart. */
export const PATCHER_SECTIONS_PANEL_LABEL: string = "Configuration";

const SECTIONS: Array<{ id: EPatcherSection; label: string; description: string; icon: ReactElement }> = [
  {
    id: EPatcherSection.COMPARISON,
    label: "Comparison",
    description: "What the patch is built from",
    icon: <CompareArrowsIcon />,
  },
  { id: EPatcherSection.OUTPUT, label: "Output", description: "Where the volumes land", icon: <FolderIcon /> },
  { id: EPatcherSection.SELECTION, label: "Selection", description: "What is compared", icon: <TuneIcon /> },
  { id: EPatcherSection.HEADER, label: "Header", description: "Where it mounts", icon: <TitleIcon /> },
  { id: EPatcherSection.OPTIONS, label: "Options", description: "How it is written", icon: <SettingsIcon /> },
];

/**
 * Navigation for the patching configuration, drawn by the shell as the tool's left panel.
 */
export function PatcherSectionsMenu({
  "data-testid": dataTestId = "patcher-sections-menu",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const patcherService: ArchivesPatcherService = useInjection(ArchivesPatcherService);

  const items: Array<IEditorSideMenuItem> = SECTIONS.map((section) => ({
    label: section.label,
    description: section.description,
    icon: section.icon,
    isSelected: patcherService.section === section.id,
    onClick: () => patcherService.openSection(section.id),
  }));

  return (
    <EditorSideMenu
      data-testid={dataTestId}
      id={id}
      className={className}
      header={<EditorPanelHeader title={PATCHER_SECTIONS_PANEL_LABEL} />}
      sections={items}
    />
  );
}
