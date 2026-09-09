import { Box } from "@mui/material";
import { ReactElement } from "react";

import { LtxInventoryFile } from "@/core/bindings/types/xrf-ltx-inspect";
import { ConfigsMenuBadges } from "@/core/ltx/components/ConfigsMenu/ConfigsMenuBadges";
import { ITreeNode } from "@/core/ui/tree/tree-node";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IConfigsTreeLabelProps extends BaseComponentProps {
  /** Node being labelled; only a leaf stands for a config and carries badges. */
  item: ITreeNode<LtxInventoryFile>;
}

/**
 * A row's name, with what the config is to the project beside it.
 */
export function ConfigsTreeLabel({ item }: IConfigsTreeLabelProps): ReactElement {
  if (!item.payload) {
    return <>{item.label}</>;
  }

  return (
    <Box sx={{ alignItems: "center", display: "flex", gap: 0.75, minWidth: 0 }}>
      <Box component={"span"} sx={{ minWidth: 0, overflow: "hidden", textOverflow: "ellipsis" }}>
        {item.label}
      </Box>

      <ConfigsMenuBadges file={item.payload} />
    </Box>
  );
}
