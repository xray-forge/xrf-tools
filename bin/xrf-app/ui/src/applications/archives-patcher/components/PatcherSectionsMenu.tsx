import { default as CompareArrowsIcon } from "@mui/icons-material/CompareArrows";
import { default as FolderIcon } from "@mui/icons-material/Folder";
import { default as SettingsIcon } from "@mui/icons-material/Settings";
import { default as TitleIcon } from "@mui/icons-material/Title";
import { default as TuneIcon } from "@mui/icons-material/Tune";
import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { EPatcherSection, PatcherService } from "@/applications/archives-patcher/services/patcher";
import { EditorPanelHeader } from "@/core/shell/editor/EditorPanelHeader";
import { EditorSideMenu, IEditorSideMenuItem } from "@/core/shell/editor/EditorSideMenu";
import { BaseComponentProps } from "@/lib/dom/element-types";

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
 * Navigation for the patching configuration, drawn by the shell as the application's left panel.
 */
export function PatcherSectionsMenu({
  "data-testid": dataTestId = "patcher-sections-menu",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const patcherService: PatcherService = useInjection(PatcherService);

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
