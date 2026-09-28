import { IRendererPoolUse, IRendererStaticDrawReport } from "@xrf/renderer";
import { ReactElement } from "react";

import { EditorPanelProperty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatCount } from "@/lib/format/number";

interface ILevelStreamStaticDrawsSectionProps extends BaseComponentProps {
  staticDraws: IRendererStaticDrawReport;
}

/** How full one static draw pool is: what it holds against what it has room for. */
function formatPoolUse({ used, capacity }: IRendererPoolUse): string {
  return `${formatCount(used)} of ${formatCount(capacity)}`;
}

/**
 * How full the static draws' pools are, what the culls kept and occlusion removed, and how often one fell back to
 * drawing plainly.
 */
export function LevelStreamStaticDrawsSection({
  "data-testid": dataTestId = "level-stream-static-draws-section",
  id,
  className,
  staticDraws,
}: ILevelStreamStaticDrawsSectionProps): ReactElement {
  const { kept, lists, occluded } = staticDraws;

  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Static draws"}>
      <EditorPanelProperty label={"Slots"} value={formatPoolUse(staticDraws.slots)} />
      <EditorPanelProperty label={"Clusters"} value={formatPoolUse(staticDraws.clusters)} />
      <EditorPanelProperty label={"Places"} value={formatPoolUse(staticDraws.places)} />
      <EditorPanelProperty label={"Instance rows"} value={formatPoolUse(staticDraws.rows)} />
      <EditorPanelProperty label={"Surface lists"} value={formatPoolUse(lists.surfaces)} />
      <EditorPanelProperty label={"Shadow lists"} value={formatPoolUse(lists.shadows)} />
      <EditorPanelProperty label={"Batch draws"} value={formatCount(staticDraws.commands)} />
      <EditorPanelProperty
        label={"Kept"}
        value={`${formatCount(kept.clusters)} clusters · ${formatCount(kept.triangles)} triangles`}
      />
      <EditorPanelProperty
        label={"Occluded"}
        value={`${formatCount(occluded.clusters)} clusters · ${formatCount(occluded.triangles)} triangles`}
      />
      <EditorPanelProperty label={"Drawn plainly at the limit"} value={staticDraws.fallbacks} />
    </EditorPanelSection>
  );
}
