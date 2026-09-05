import { Box } from "@mui/material";
import { ReactElement } from "react";

import { TextureBadgeMarks } from "@/applications/textures-explorer/components/editor/tree/TextureBadgeMarks";
import { ITextureNode } from "@/applications/textures-explorer/lib/texture-catalog";
import { ITreeNode } from "@/core/ui/tree/tree-node";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ITextureTreeLabelProps extends BaseComponentProps {
  /** Node being labelled; only a leaf stands for a texture and carries marks. */
  item: ITreeNode<ITextureNode>;
}

/**
 * A row's name, with what the texture is marked beside it.
 */
export function TextureTreeLabel({ item }: ITextureTreeLabelProps): ReactElement {
  if (!item.payload) {
    return <>{item.label}</>;
  }

  return (
    <Box sx={{ alignItems: "center", display: "flex", gap: 0.75, minWidth: 0 }}>
      <Box component={"span"} sx={{ minWidth: 0, overflow: "hidden", textOverflow: "ellipsis" }}>
        {item.label}
      </Box>

      <TextureBadgeMarks badges={item.payload.badges} />
    </Box>
  );
}
