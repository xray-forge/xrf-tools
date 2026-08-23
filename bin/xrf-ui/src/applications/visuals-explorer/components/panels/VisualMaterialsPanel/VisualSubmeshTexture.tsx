import { Chip } from "@mui/material";
import { ReactElement } from "react";

import {
  describeResolution,
  describeTextureState,
  IVisualTextureStateDescriptor,
} from "@/applications/visuals-explorer/components/panels/VisualMaterialsPanel/VisualSubmeshTexture.utils";
import { VisualSubmeshTextureSource } from "@/applications/visuals-explorer/components/panels/VisualMaterialsPanel/VisualSubmeshTextureSource";
import { VisualPanelRow } from "@/applications/visuals-explorer/components/panels/VisualPanelRow";
import { XrayAsset } from "@/core/bindings/types/xrf-vfs";
import { VisualTextureDependency } from "@/core/bindings/types/xrf-visual";
import { EVisualTextureState, getLocatedAsset, IVisualTextureStatus } from "@/core/visuals/lib/visual-texture";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

export interface IVisualSubmeshTextureProps extends BaseComponentProps {
  texture: Nullable<VisualTextureDependency>;
  status: Nullable<IVisualTextureStatus>;
}

/**
 * What became of one submesh's texture: the outcome, the root that answered, and the file inside it.
 */
export function VisualSubmeshTexture({ texture, status }: IVisualSubmeshTextureProps): ReactElement | null {
  if (!texture) {
    return null;
  }

  const { resolution } = texture;
  const located: Nullable<XrayAsset> = getLocatedAsset(resolution);
  const state: EVisualTextureState = status?.state ?? EVisualTextureState.ABSENT;
  const descriptor: IVisualTextureStateDescriptor = describeTextureState(state);

  return (
    <>
      <VisualPanelRow
        label={"Texture"}
        value={<Chip size={"small"} color={descriptor.color} variant={"outlined"} label={descriptor.label} />}
      />
      <VisualPanelRow label={"Resolution"} value={describeResolution(resolution)} />

      {located ? <VisualSubmeshTextureSource asset={located} /> : null}

      {resolution.kind === "missing"
        ? resolution.roots.map((root: string) => <VisualPanelRow key={root} label={"Searched"} value={root} />)
        : null}

      {resolution.kind === "rejected" ? <VisualPanelRow label={"Rejected"} value={resolution.reason} /> : null}

      {status?.reason ? <VisualPanelRow label={"Texture error"} value={status.reason} /> : null}
    </>
  );
}
