import { Box, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useMemo } from "react";

import { AssetTextureDescriptor } from "@/core/bindings/types/xrf-app";
import { VisualDescription, VisualTextureDependency } from "@/core/bindings/types/xrf-visual";
import { IVisualInspection, VISUAL_INSPECTION } from "@/core/visuals/components/panels/visual-inspection";
import { VisualPanel } from "@/core/visuals/components/panels/VisualPanel";
import { VisualPanelEmpty } from "@/core/visuals/components/panels/VisualPanelEmpty";
import {
  describeVisualTextureSummary,
  IVisualTextureSummary,
  summarizeVisualTextures,
} from "@/core/visuals/lib/visual-texture-summary";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import { VisualSubmeshSection } from "./VisualSubmeshSection";

/** Every submesh of the open visual, in the order the file stores them. */
export interface IVisualMaterialsPanelProps extends BaseComponentProps {}

export function VisualMaterialsPanel({
  "data-testid": dataTestId = "visual-materials-panel",
  id,
  className,
}: IVisualMaterialsPanelProps = {}): ReactElement {
  const { selected, textureStatuses }: IVisualInspection = useInjection(VISUAL_INSPECTION);
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
      <VisualPanel data-testid={dataTestId} id={id} className={className} title={"Materials"}>
        <VisualPanelEmpty label={"No materials. Texture and shader names per child visual."} />
      </VisualPanel>
    );
  }

  return (
    <VisualPanel data-testid={dataTestId} id={id} className={className} title={"Materials"}>
      {summary ? (
        <Box sx={{ paddingX: 2, paddingY: 1, borderBottom: 1, borderColor: "divider" }}>
          <Typography variant={"body2"} sx={{ color: "text.secondary" }}>
            {describeVisualTextureSummary(summary)}
          </Typography>

          {selected?.texturesLtx ? (
            <Typography variant={"caption"} sx={{ display: "block", marginTop: 0.5, color: "warning.main" }}>
              {`Bump declarations are read from .thm files only. ${selected.texturesLtx.logicalPath} may declare ` +
                "more, and is not read."}
            </Typography>
          ) : null}
        </Box>
      ) : null}

      {description.submeshes.map((submesh, index) => (
        <VisualSubmeshSection
          key={submesh.index}
          submesh={submesh}
          isFirst={index === 0}
          texture={textures.get(submesh.index) ?? null}
          status={textureStatuses.get(submesh.index) ?? null}
          textures={described}
          materials={selected?.materials}
        />
      ))}
    </VisualPanel>
  );
}
