import { Box, Chip } from "@mui/material";
import { ReactElement } from "react";

import { AssetTextureDescriptor } from "@/core/bindings/types/xrf-app";
import { XrayMaterialDescriptor } from "@/core/bindings/types/xrf-material";
import { VisualSubmesh, VisualTextureDependency } from "@/core/bindings/types/xrf-visual";
import { VisualPanelRow } from "@/core/visuals/components/panels/VisualPanelRow";
import { VisualPanelSection } from "@/core/visuals/components/panels/VisualPanelSection";
import { IVisualTextureStatus } from "@/core/visuals/lib/visual-texture";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { ABSENT_VALUE } from "@/lib/format/number";
import { Nullable } from "@/lib/types/general";

import { VisualSubmeshMaterial } from "./VisualSubmeshMaterial";
import { VisualSubmeshTexture } from "./VisualSubmeshTexture";

export interface IVisualSubmeshSectionProps extends BaseComponentProps {
  submesh: VisualSubmesh;
  isFirst: boolean;
  /** What the backend resolved this submesh's reference to, absent when the model reports no textures at all. */
  texture?: Nullable<VisualTextureDependency>;
  /** What the frontend then did with it. */
  status?: Nullable<IVisualTextureStatus>;
  /** Descriptors the open reported, keyed by logical path. */
  textures?: Record<string, AssetTextureDescriptor>;
  /** What the renderer builds for each declared texture, keyed by the reference as the mesh declares it. */
  materials?: Record<string, XrayMaterialDescriptor>;
}

/**
 * One submesh: what it is textured with, how much of it there is, or why there is none of it.
 */
export function VisualSubmeshSection({
  "data-testid": dataTestId = "visual-submesh-section",
  id,
  className,
  submesh,
  isFirst,
  texture = null,
  status = null,
  textures,
  materials,
}: IVisualSubmeshSectionProps): ReactElement {
  const { content } = submesh;

  return (
    <VisualPanelSection
      data-testid={dataTestId}
      id={id}
      className={className}
      isFirst={isFirst}
      title={
        <Box sx={{ display: "flex", alignItems: "center", gap: 0.75 }}>
          <Box component={"span"} sx={{ minWidth: 0, wordBreak: "break-all" }}>
            {submesh.textureName ?? `Submesh ${submesh.index}`}
          </Box>

          {content.kind === "skipped" ? (
            <Chip size={"small"} color={"warning"} variant={"outlined"} label={content.cause} />
          ) : null}
        </Box>
      }
    >
      <VisualPanelRow label={"Shader"} value={submesh.shaderName ?? ABSENT_VALUE} />

      <VisualPanelRow label={"Type"} value={submesh.modelTypeLabel} />

      <VisualSubmeshTexture texture={texture} status={status} textures={textures} />

      <VisualSubmeshMaterial material={texture ? (materials?.[texture.reference] ?? null) : null} />

      {content.kind === "packed" ? (
        <>
          <VisualPanelRow label={"Vertices"} value={content.geometry.vertexCount} />

          <VisualPanelRow
            label={"Triangles"}
            value={`${content.geometry.detailLevels[0].count / 3} of ${content.geometry.indexCount / 3}`}
          />

          {content.geometry.detailLevels.length > 1 ? (
            <VisualPanelRow label={"Detail levels"} value={content.geometry.detailLevels.length} />
          ) : null}
        </>
      ) : (
        <VisualPanelRow label={"Reason"} value={content.reason} />
      )}
    </VisualPanelSection>
  );
}
