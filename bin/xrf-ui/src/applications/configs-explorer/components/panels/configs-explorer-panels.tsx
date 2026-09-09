import { default as DescriptionIcon } from "@mui/icons-material/Description";

import { ConfigsMenuPanel } from "@/core/ltx/components/panels/ConfigsMenuPanel";
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
  ];
}
