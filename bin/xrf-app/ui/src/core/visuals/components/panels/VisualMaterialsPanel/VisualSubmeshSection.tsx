import { ABSENT_VALUE } from "@xrf/format";
import { AssetTextureDescriptor } from "@xrf/ipc/types/xrf-app-core";
import { XrayMaterialDescriptor, XraySurfaceDescriptor } from "@xrf/ipc/types/xrf-material";
import { VisualSubmesh, VisualTextureDependency } from "@xrf/ipc/types/xrf-visual";
import { Nullable } from "@xrf/types";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Chip } from "@xrf/ui/material";
import { ReactElement } from "react";

import { IVisualBumpStatus } from "@/core/visuals/lib/visual-bump";
import { IVisualTextureStatus } from "@/core/visuals/lib/visual-texture";

import { VisualSubmeshMaterial } from "./VisualSubmeshMaterial";
import { VisualSubmeshSurface } from "./VisualSubmeshSurface";
import { VisualSubmeshTexture } from "./VisualSubmeshTexture";

interface IVisualSubmeshSectionProps extends BaseComponentProps {
  submesh: VisualSubmesh;
  isFirst: boolean;
  /** What the backend resolved this submesh's reference to, absent when the model reports no textures at all. */
  texture?: Nullable<VisualTextureDependency>;
  /** What the frontend then did with it. */
  status?: Nullable<IVisualTextureStatus>;
  /** What the frontend did with the bump pair, absent for a material that bound none. */
  bumpStatus?: Nullable<IVisualBumpStatus>;
  /** Descriptors the open reported, keyed by logical path. */
  textures?: Record<string, AssetTextureDescriptor>;
  /** What the renderer builds for each declared texture, keyed by the reference as the mesh declares it. */
  materials?: Record<string, XrayMaterialDescriptor>;
  /** How the renderer draws this submesh, from the answer the open resolved for it. */
  surface?: Nullable<XraySurfaceDescriptor>;
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
  bumpStatus = null,
  textures,
  materials,
  surface = null,
}: IVisualSubmeshSectionProps): ReactElement {
  const { content } = submesh;

  return (
    <EditorPanelSection
      data-testid={dataTestId}
      id={id}
      className={className}
      isFirst={isFirst}
      title={
        <div className={"flex items-center gap-1.5"}>
          <span className={"min-w-0 break-all"}>{submesh.textureName ?? `Submesh ${submesh.index}`}</span>

          {content.kind === "skipped" ? (
            <Chip size={"small"} color={"warning"} variant={"outlined"} label={content.cause} />
          ) : null}
        </div>
      }
    >
      <EditorPanelProperty label={"Shader"} value={submesh.shaderName ?? ABSENT_VALUE} isMonospace />

      <VisualSubmeshSurface surface={surface} />

      <EditorPanelProperty label={"Type"} value={submesh.modelTypeLabel} />

      <VisualSubmeshTexture texture={texture} status={status} textures={textures} />

      <VisualSubmeshMaterial material={texture ? (materials?.[texture.reference] ?? null) : null} status={bumpStatus} />

      {content.kind === "packed" ? (
        <>
          <EditorPanelProperty label={"Vertices"} value={content.geometry.vertexCount} />

          <EditorPanelProperty
            label={"Triangles"}
            value={`${content.geometry.detailLevels[0].count / 3} of ${content.geometry.indexCount / 3}`}
          />

          {content.geometry.detailLevels.length > 1 ? (
            <EditorPanelProperty label={"Detail levels"} value={content.geometry.detailLevels.length} />
          ) : null}
        </>
      ) : (
        <EditorPanelProperty label={"Reason"} value={content.reason} />
      )}
    </EditorPanelSection>
  );
}
