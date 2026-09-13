import { Box, Tooltip, Typography } from "@mui/material";
import { ReactElement } from "react";

import { LtxInventoryFile } from "@/core/bindings/types/xrf-ltx-inspect";
import { ITreeNode } from "@/core/ui/tree/tree-node";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IConfigsTreeLabelProps extends BaseComponentProps {
  /** Node being labelled; only a leaf stands for a config and can have come from a volume. */
  item: ITreeNode<LtxInventoryFile>;
}

/**
 * A row's name, and whether the engine reads it out of an archive.
 */
export function ConfigsTreeLabel({ item }: IConfigsTreeLabelProps): ReactElement {
  if (!item.payload || item.payload.isPhysical) {
    return <>{item.label}</>;
  }

  return (
    <Box sx={{ alignItems: "center", display: "flex", gap: 0.75, minWidth: 0 }}>
      <Box component={"span"} sx={{ minWidth: 0, overflow: "hidden", textOverflow: "ellipsis" }}>
        {item.label}
      </Box>

      <Tooltip title={"Read from an archive volume; nothing can write to it in place"}>
        <Typography
          component={"span"}
          variant={"caption"}
          sx={{ color: "text.secondary", flexShrink: 0, opacity: 0.75 }}
        >
          db
        </Typography>
      </Tooltip>
    </Box>
  );
}
