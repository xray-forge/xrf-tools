import { ReactElement } from "react";

import { ArchiveThmMaterial } from "@/core/ipc/types/xrf-app";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatNumber } from "@/lib/format/number";
import { Nullable } from "@/lib/types/general";

import { NOT_DECLARED } from "../ArchiveDescriptionPreview.utils";
import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";

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
          <ArchiveDescriptionRow label={"Material"} value={material.label} />

          <ArchiveDescriptionRow
            label={"Weight"}
            value={formatNumber(material.weight, 3)}
            caption={"Where the surface sits between the two lighting models"}
          />
        </>
      ) : (
        <ArchiveDescriptionRow label={"Material"} value={NOT_DECLARED} />
      )}
    </EditorPanelSection>
  );
}
