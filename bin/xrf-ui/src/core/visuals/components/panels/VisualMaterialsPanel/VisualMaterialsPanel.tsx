import { Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useMemo } from "react";

import { AssetTextureDescriptor } from "@/core/ipc/types/xrf-app";
import { VisualDescription, VisualTextureDependency } from "@/core/ipc/types/xrf-visual";
import { EditorPanel, EditorPanelEmpty } from "@/core/shell/editor/EditorPanel";
import { IVisualInspection, VISUAL_INSPECTION } from "@/core/visuals/components/panels/visual-inspection";
import {
  describeVisualTextureSummary,
  IVisualTextureSummary,
  summarizeVisualTextures,
} from "@/core/visuals/lib/visual-texture-summary";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import { VisualSubmeshSection } from "./VisualSubmeshSection";

export function VisualMaterialsPanel({
  "data-testid": dataTestId = "visual-materials-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const { selected, textureStatuses, bumpStatuses }: IVisualInspection = useInjection(VISUAL_INSPECTION);
  const description: Nullable<VisualDescription> = selected?.description ?? null;
  const described: Record<string, AssetTextureDescriptor> = selected?.textures ?? {};

  /**
   * What the model's textures weigh, or null when it declares none.
   */
  const summary: Nullable<IVisualTextureSummary> = useMemo(
    () =>
      selected?.dependencies.textures.length
        ? summarizeVisualTextures(selected.textures, selected.dependencies.textures, selected.materials)
        : null,
    [selected]
  );

  const textures: ReadonlyMap<number, VisualTextureDependency> = useMemo(
    () =>
      new Map(
        (selected?.dependencies.textures ?? []).map((it: VisualTextureDependency) => [it.submeshIndex, it] as const)
      ),
    [selected]
  );

  if (!description || description.submeshes.length === 0) {
    return (
      <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Materials"}>
        <EditorPanelEmpty label={"No materials. Texture and shader names per child visual."} />
      </EditorPanel>
    );
  }

  return (
    <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Materials"}>
      {summary ? (
        <div className={"border-b border-divider px-4 py-2"}>
          <Typography className={"text-text-secondary"} variant={"body2"}>
            {describeVisualTextureSummary(summary)}
          </Typography>

          {selected?.texturesLtx ? (
            <Typography className={"mt-1 block text-warning"} variant={"caption"}>
              {`Bump declarations are read from .thm files only. ${selected.texturesLtx.logicalPath} may declare ` +
                "more, and is not read."}
            </Typography>
          ) : null}
        </div>
      ) : null}

      {description.submeshes.map((submesh, index) => (
        <VisualSubmeshSection
          key={submesh.index}
          submesh={submesh}
          isFirst={index === 0}
          texture={textures.get(submesh.index) ?? null}
          status={textureStatuses.get(submesh.index) ?? null}
          bumpStatus={bumpStatuses.get(submesh.index) ?? null}
          textures={described}
          materials={selected?.materials}
          surfaces={selected?.surfaces}
        />
      ))}
    </EditorPanel>
  );
}
