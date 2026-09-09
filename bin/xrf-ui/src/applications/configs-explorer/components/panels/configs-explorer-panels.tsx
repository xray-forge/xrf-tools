import { default as DescriptionIcon } from "@mui/icons-material/Description";
import { default as ListIcon } from "@mui/icons-material/FormatListBulleted";

import { ConfigsMenuPanel } from "@/core/ltx/components/panels/ConfigsMenuPanel";
import { ConfigsSectionsPanel } from "@/core/ltx/components/panels/ConfigsSectionsPanel";
import { IEditorPanel } from "@/core/shell/editor-shell";

/**
 * The panels the explorer offers.
 *
 * @returns The panels to register.
 */
export function createConfigsExplorerPanels(): Array<IEditorPanel> {
  return [
    {
      icon: <DescriptionIcon />,
      id: "configs",
      isOpenByDefault: true,
      label: "Configs",
      render: () => <ConfigsMenuPanel />,
      side: "left" as const,
    },
    {
      icon: <ListIcon />,
      id: "sections",
      isOpenByDefault: true,
      label: "Sections",
      render: () => <ConfigsSectionsPanel />,
    },
  ];
}
