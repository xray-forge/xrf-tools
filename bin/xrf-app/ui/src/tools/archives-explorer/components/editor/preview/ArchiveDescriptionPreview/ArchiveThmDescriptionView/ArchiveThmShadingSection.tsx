import { formatNumber } from "@xrf/format";
import { ArchiveThmMaterial } from "@xrf/ipc/types/xrf-app-archives";
import { Nullable } from "@xrf/types";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { NOT_DECLARED } from "../ArchiveDescriptionPreview.utils";

interface IArchiveThmShadingSectionProps extends BaseComponentProps {
  material: Nullable<ArchiveThmMaterial>;
}

/**
 * The one piece of authoring data that reaches the renderer through the descriptor rather than through the DDS.
 */
export function ArchiveThmShadingSection({
  "data-testid": dataTestId = "archive-thm-shading-section",
  id,
  className,
  material,
}: IArchiveThmShadingSectionProps): ReactElement {
  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Shading"}>
      {material ? (
        <>
          <EditorPanelProperty isInline label={"Material"} value={material.label} />

          <EditorPanelProperty
            isInline
            label={"Weight"}
            value={formatNumber(material.weight, 3)}
            caption={"Where the surface sits between the two lighting models"}
          />
        </>
      ) : (
        <EditorPanelProperty isInline label={"Material"} value={NOT_DECLARED} />
      )}
    </EditorPanelSection>
  );
}
