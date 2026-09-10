import { default as DescriptionIcon } from "@mui/icons-material/Description";
import { default as ListIcon } from "@mui/icons-material/FormatListBulleted";
import { default as RuleIcon } from "@mui/icons-material/Rule";
import { default as WarningIcon } from "@mui/icons-material/WarningAmber";

import { ConfigsMenuPanel } from "@/core/ltx/components/panels/ConfigsMenuPanel";
import { ConfigsProblemsPanel } from "@/core/ltx/components/panels/ConfigsProblemsPanel";
import { ConfigsSchemePanel } from "@/core/ltx/components/panels/ConfigsSchemePanel";
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
    {
      icon: <RuleIcon />,
      id: "scheme",
      label: "Scheme",
      render: () => <ConfigsSchemePanel />,
    },
    {
      icon: <WarningIcon />,
      id: "problems",
      label: "Problems",
      render: () => <ConfigsProblemsPanel />,
    },
  ];
}
