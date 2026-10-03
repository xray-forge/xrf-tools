import { ReactElement } from "react";

import { RenderPoolUse, RenderStaticReport } from "@/core/ipc/types/xrf-renderer";
import { EditorPanelProperty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatCount } from "@/lib/format/number";

interface ILevelRendererStaticDrawsSectionProps extends BaseComponentProps {
  staticDraws: RenderStaticReport;
}

/** How full one static draw pool is: what it holds against what it has room for. */
function formatPoolUse({ used, capacity }: RenderPoolUse): string {
  return `${formatCount(used)} of ${formatCount(capacity)}`;
}

/**
 * How full the static draws' pools are, and what the camera's cull kept and occlusion removed.
 */
export function LevelRendererStaticDrawsSection({
  "data-testid": dataTestId = "level-renderer-static-draws-section",
  id,
  className,
  staticDraws,
}: ILevelRendererStaticDrawsSectionProps): ReactElement {
  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Static draws"}>
      <EditorPanelProperty label={"Slots"} value={formatPoolUse(staticDraws.slots)} />
      <EditorPanelProperty label={"Clusters"} value={formatPoolUse(staticDraws.clusters)} />
      <EditorPanelProperty label={"Places"} value={formatPoolUse(staticDraws.places)} />
      <EditorPanelProperty label={"Instance rows"} value={formatPoolUse(staticDraws.rows)} />
      <EditorPanelProperty label={"Impostors"} value={formatPoolUse(staticDraws.lods)} />
      <EditorPanelProperty label={"Surface list"} value={formatPoolUse(staticDraws.surfaceList)} />
      <EditorPanelProperty label={"Batch draws"} value={formatCount(staticDraws.commands)} />
      <EditorPanelProperty
        label={"Kept"}
        value={`${formatCount(staticDraws.keptClusters)} clusters · ${formatCount(staticDraws.keptTriangles)} triangles`}
      />
      <EditorPanelProperty
        label={"Occluded"}
        value={
          `${formatCount(staticDraws.occludedClusters)} clusters · ` +
          `${formatCount(staticDraws.occludedTriangles)} triangles`
        }
      />
    </EditorPanelSection>
  );
}
