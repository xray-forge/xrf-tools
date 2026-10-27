import { BaseComponentProps } from "@xrf/ui/element-types";
import { ITreeNode } from "@xrf/ui/tree";
import { ReactElement } from "react";

import { TextureBadgeMarks } from "@/applications/textures-explorer/components/tree/TextureBadgeMarks";
import { ITextureNode } from "@/applications/textures-explorer/lib/texture-catalog";

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
    <div className={"flex min-w-0 items-center gap-1.5"}>
      <span className={"min-w-0 overflow-hidden text-ellipsis"}>{item.label}</span>

      <TextureBadgeMarks badges={item.payload.badges} />
    </div>
  );
}
