import { useInjection } from "@wirestate/react";
import { ReactElement, useMemo } from "react";

import { VisualSubmeshSection } from "@/applications/visuals-explorer/components/panels/VisualMaterialsPanel/VisualSubmeshSection";
import { VisualPanel } from "@/applications/visuals-explorer/components/panels/VisualPanel";
import { VisualPanelEmpty } from "@/applications/visuals-explorer/components/panels/VisualPanelEmpty";
import { VisualsService } from "@/applications/visuals-explorer/services/visuals";
import { VisualDescription, VisualTextureDependency } from "@/core/bindings/types/xrf-visual";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

/** Every submesh of the open visual, in the order the file stores them. */
export interface IVisualMaterialsPanelProps extends BaseComponentProps {}

export function VisualMaterialsPanel({
  "data-testid": dataTestId = "visual-materials-panel",
  id,
  className,
}: IVisualMaterialsPanelProps = {}): ReactElement {
  const visualsService: VisualsService = useInjection(VisualsService);
  const description: Nullable<VisualDescription> = visualsService.visual.value?.selected.description ?? null;

  const textures: ReadonlyMap<number, VisualTextureDependency> = useMemo(
    () =>
      new Map(
        (visualsService.visual.value?.selected.dependencies.textures ?? []).map(
          (it: VisualTextureDependency) => [it.submeshIndex, it] as const
        )
      ),
    [visualsService.visual.value]
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
      {description.submeshes.map((submesh, index) => (
        <VisualSubmeshSection
          key={submesh.index}
          submesh={submesh}
          isFirst={index === 0}
          texture={textures.get(submesh.index) ?? null}
          status={visualsService.textureStatuses.get(submesh.index) ?? null}
        />
      ))}
    </VisualPanel>
  );
}
