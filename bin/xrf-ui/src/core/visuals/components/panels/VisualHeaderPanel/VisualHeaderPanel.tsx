import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { VisualDescription } from "@/core/bindings/types/xrf-visual";
import { EditorPanel, EditorPanelEmpty, EditorPanelRow, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { VISUAL_INSPECTION } from "@/core/visuals/components/panels/visual-inspection";
import { VisualBoundsSection } from "@/core/visuals/components/panels/VisualHeaderPanel/VisualBoundsSection";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { ABSENT_VALUE } from "@/lib/format/number";
import { Nullable } from "@/lib/types/general";

export function VisualHeaderPanel({
  "data-testid": dataTestId = "visual-header-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const description: Nullable<VisualDescription> = useInjection(VISUAL_INSPECTION).selected?.description ?? null;

  if (!description) {
    return (
      <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Header"}>
        <EditorPanelEmpty label={"No visual open. Open an ogf file to see its header."} />
      </EditorPanel>
    );
  }

  return (
    <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Header"}>
      <EditorPanelSection title={"Model"} isFirst>
        <EditorPanelRow label={"Format version"} value={description.version} />
        <EditorPanelRow label={"Type"} value={description.modelTypeLabel} />
        <EditorPanelRow label={"Type id"} value={description.modelType} />
        <EditorPanelRow label={"Shader id"} value={description.shaderId} />
        <EditorPanelRow label={"Submeshes"} value={description.submeshes.length} />
      </EditorPanelSection>

      <EditorPanelSection title={"Source"}>
        <EditorPanelRow label={"Built from"} value={description.sourceFile ?? ABSENT_VALUE} isMonospace />
      </EditorPanelSection>

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
    </EditorPanel>
  );
}
