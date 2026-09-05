import { Chip } from "@mui/material";
import { ReactElement } from "react";

import { describeResolution, getLocatedAsset } from "@/core/assets/lib/resolution";
import { AssetTextureDescriptor } from "@/core/bindings/types/xrf-app";
import { XrayAsset } from "@/core/bindings/types/xrf-vfs";
import { VisualTextureDependency } from "@/core/bindings/types/xrf-visual";
import { VisualPanelRow } from "@/core/visuals/components/panels/VisualPanelRow";
import { EVisualTextureState, IVisualTextureStatus } from "@/core/visuals/lib/visual-texture";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import { describeTextureState, IVisualTextureStateDescriptor } from "./VisualSubmeshTexture.utils";
import { VisualSubmeshTextureSource } from "./VisualSubmeshTextureSource";

export interface IVisualSubmeshTextureProps extends BaseComponentProps {
  texture: Nullable<VisualTextureDependency>;
  status: Nullable<IVisualTextureStatus>;
  /** Descriptors the open reported, keyed by logical path, so a file two submeshes share is described once. */
  textures?: Record<string, AssetTextureDescriptor>;
}

/**
 * What became of one submesh's texture: the outcome, the root that answered, the file inside it, and what that file is.
 */
export function VisualSubmeshTexture({ texture, status, textures }: IVisualSubmeshTextureProps): ReactElement | null {
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

      {located ? (
        <VisualSubmeshTextureSource asset={located} descriptor={textures?.[located.logicalPath] ?? null} />
      ) : null}

      {resolution.kind === "missing"
        ? resolution.roots.map((root: string) => <VisualPanelRow key={root} label={"Searched"} value={root} />)
        : null}

      {resolution.kind === "rejected" ? <VisualPanelRow label={"Rejected"} value={resolution.reason} /> : null}

      {state === EVisualTextureState.DECODED ? (
        <VisualPanelRow
          label={"Upload"}
          value={"Expanded from a layout the renderer cannot read, so it is uploaded without a mip chain"}
        />
      ) : null}

      {status?.reason ? <VisualPanelRow label={"Texture error"} value={status.reason} /> : null}
    </>
  );
}
