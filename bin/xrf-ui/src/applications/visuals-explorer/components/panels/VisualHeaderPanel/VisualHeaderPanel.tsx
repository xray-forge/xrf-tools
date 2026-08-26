import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { VisualBoundsSection } from "@/applications/visuals-explorer/components/panels/VisualHeaderPanel/VisualBoundsSection";
import { VisualsService } from "@/applications/visuals-explorer/services/visuals";
import { VisualDescription } from "@/core/bindings/types/xrf-visual";
import { VisualPanel } from "@/core/visuals/components/panels/VisualPanel";
import { VisualPanelEmpty } from "@/core/visuals/components/panels/VisualPanelEmpty";
import { VisualPanelRow } from "@/core/visuals/components/panels/VisualPanelRow";
import { VisualPanelSection } from "@/core/visuals/components/panels/VisualPanelSection";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { ABSENT_VALUE } from "@/lib/format/number";
import { Nullable } from "@/lib/types/general";

/**
 * What the file says it is, with both extents side by side.
 *
 * Declared and measured bounds are shown together and unreconciled: a file whose header disagrees with its own geometry
 * is worth seeing rather than having one quietly stand in for the other. Each is captioned so they cannot be misread as
 * one list of eight numbers.
 */
export interface IVisualHeaderPanelProps extends BaseComponentProps {}

export function VisualHeaderPanel({
  "data-testid": dataTestId = "visual-header-panel",
  id,
  className,
}: IVisualHeaderPanelProps = {}): ReactElement {
  const visualsService: VisualsService = useInjection(VisualsService);
  const description: Nullable<VisualDescription> = visualsService.selected?.description ?? null;

  if (!description) {
    return (
      <VisualPanel data-testid={dataTestId} id={id} className={className} title={"Header"}>
        <VisualPanelEmpty label={"No visual open. Open an ogf file to see its header."} />
      </VisualPanel>
    );
  }

  return (
    <VisualPanel data-testid={dataTestId} id={id} className={className} title={"Header"}>
      <VisualPanelSection title={"Model"} isFirst>
        <VisualPanelRow label={"Format version"} value={description.version} />
        <VisualPanelRow label={"Type"} value={description.modelTypeLabel} />
        <VisualPanelRow label={"Type id"} value={description.modelType} />
        <VisualPanelRow label={"Shader id"} value={description.shaderId} />
        <VisualPanelRow label={"Submeshes"} value={description.submeshes.length} />
      </VisualPanelSection>

      <VisualPanelSection title={"Source"}>
        <VisualPanelRow label={"Built from"} value={description.sourceFile ?? ABSENT_VALUE} />
      </VisualPanelSection>

      <VisualBoundsSection
        title={"Declared bounds"}
        caption={"As the header states them"}
        bounds={description.declaredBounds}
      />

      <VisualBoundsSection
        title={"Measured bounds"}
        caption={"As the drawn geometry spans"}
        bounds={description.computedBounds}
      />
    </VisualPanel>
  );
}
