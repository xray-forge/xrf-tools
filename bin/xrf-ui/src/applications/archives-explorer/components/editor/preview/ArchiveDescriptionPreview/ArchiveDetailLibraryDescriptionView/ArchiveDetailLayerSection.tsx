import { ReactElement } from "react";

import { ArchiveDetailLibraryDescription } from "@/core/ipc/types/xrf-app";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { formatCount } from "../ArchiveDescriptionPreview.utils";
import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";
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
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Detail layer"} isFirst>
      <ArchiveDescriptionRow
        label={"Planted"}
        value={describePlanting(description)}
        caption={`Of ${formatCount(description.slots)} slots, each ${DETAIL_SLOT_METERS} m square`}
      />

      <ArchiveDescriptionRow
        label={"Grid"}
        value={`${description.sizeX} × ${description.sizeZ}`}
        caption={describeCoverage(description)}
      />

      <ArchiveDescriptionRow
        label={"Objects"}
        value={`${description.entries.length}`}
        caption={"A slot addresses one of these with six bits, so a library holds at most 63"}
      />

      <ArchiveDescriptionRow label={"Version"} value={`${description.version}`} />
    </EditorPanelSection>
  );
}
