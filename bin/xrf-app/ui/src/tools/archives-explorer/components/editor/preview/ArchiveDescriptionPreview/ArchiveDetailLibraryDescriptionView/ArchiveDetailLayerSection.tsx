import { formatCount } from "@xrf/format";
import { ArchiveDetailLibraryDescription } from "@xrf/ipc/types/xrf-app-archives";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { describeCoverage, describePlanting, DETAIL_SLOT_METERS } from "./ArchiveDetailLibraryDescriptionView.utils";

interface IArchiveDetailLayerSectionProps extends BaseComponentProps {
  description: ArchiveDetailLibraryDescription;
}

/**
 * What the detail layer covers, taken over the whole grid.
 */
export function ArchiveDetailLayerSection({
  "data-testid": dataTestId = "archive-detail-layer-section",
  id,
  className,
  description,
}: IArchiveDetailLayerSectionProps): ReactElement {
  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Detail layer"}>
      <EditorPanelProperty
        isInline
        label={"Planted"}
        value={describePlanting(description)}
        caption={`Of ${formatCount(description.slots)} slots, each ${DETAIL_SLOT_METERS} m square`}
      />

      <EditorPanelProperty
        isInline
        label={"Grid"}
        value={`${description.sizeX} × ${description.sizeZ}`}
        caption={describeCoverage(description)}
      />

      <EditorPanelProperty
        isInline
        label={"Objects"}
        value={`${description.entries.length}`}
        caption={"A slot addresses one of these with six bits, so a library holds at most 63"}
      />

      <EditorPanelProperty isInline label={"Version"} value={`${description.version}`} />
    </EditorPanelSection>
  );
}
