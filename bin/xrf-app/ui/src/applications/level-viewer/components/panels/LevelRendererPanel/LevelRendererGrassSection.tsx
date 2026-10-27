import { formatCount } from "@xrf/format";
import { RenderGrassReport } from "@xrf/ipc/types/xrf-renderer";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

interface ILevelRendererGrassSectionProps extends BaseComponentProps {
  grass: RenderGrassReport;
}

/**
 * What the grass came to on a recent frame: the tufts its cull kept, against the most its lists hold.
 */
export function LevelRendererGrassSection({
  "data-testid": dataTestId = "level-renderer-grass-section",
  id,
  className,
  grass,
}: ILevelRendererGrassSectionProps): ReactElement {
  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Grass"}>
      <EditorPanelProperty
        label={"Drawn"}
        value={
          grass.drawn > grass.capacity
            ? `${formatCount(grass.capacity)} tufts, the lists full: ${formatCount(grass.drawn)} kept`
            : `${formatCount(grass.drawn)} tufts`
        }
      />
      <EditorPanelProperty label={"Lists hold"} value={`${formatCount(grass.capacity)} tufts`} />
    </EditorPanelSection>
  );
}
