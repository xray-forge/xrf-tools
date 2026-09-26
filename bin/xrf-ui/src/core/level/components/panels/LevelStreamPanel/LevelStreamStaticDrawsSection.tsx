import { IRendererPoolUse, IRendererStaticDrawReport } from "@xrf/renderer";
import { ReactElement } from "react";

import { EditorPanelProperty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { formatCount } from "@/lib/format/number";

interface ILevelStreamStaticDrawsSectionProps {
  staticDraws: IRendererStaticDrawReport;
}

/** How full one static draw pool is: what it holds against what it has room for. */
function formatPoolUse({ used, capacity }: IRendererPoolUse): string {
  return `${formatCount(used)} of ${formatCount(capacity)}`;
}

/**
 * How full the static draws' pools are, what occlusion removed, and how often one fell back to drawing plainly.
 */
export function LevelStreamStaticDrawsSection({ staticDraws }: ILevelStreamStaticDrawsSectionProps): ReactElement {
  const { occluded } = staticDraws;

  return (
    <EditorPanelSection title={"Static draws"}>
      <EditorPanelProperty label={"Slots"} value={formatPoolUse(staticDraws.slots)} />
      <EditorPanelProperty label={"Instanced places"} value={formatPoolUse(staticDraws.places)} />
      <EditorPanelProperty label={"Instance rows"} value={formatPoolUse(staticDraws.rows)} />
      <EditorPanelProperty
        label={"Occluded"}
        value={
          `${formatCount(occluded.draws)} draws · ${formatCount(occluded.instances)} instances · ` +
          `${formatCount(occluded.triangles)} triangles`
        }
      />
      <EditorPanelProperty label={"Drawn plainly at the limit"} value={staticDraws.fallbacks} />
    </EditorPanelSection>
  );
}
