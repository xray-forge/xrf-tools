import { ReactElement } from "react";

import { ArchiveLevelSomDescription } from "@/core/ipc/types/xrf-app";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchiveDescriptionLayout } from "../ArchiveDescriptionLayout";
import { formatCount, formatLevelBounds } from "../ArchiveDescriptionPreview.utils";
import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";
import { describeOcclusionRange, describeTwoSided } from "./ArchiveLevelOcclusionView.utils";

interface IArchiveLevelSomViewProps extends BaseComponentProps {
  description: ArchiveLevelSomDescription;
}

/**
 * A level's sound occlusion mesh: the geometry that muffles what is heard through it.
 */
export function ArchiveLevelSomView({
  "data-testid": dataTestId = "archive-level-som-view",
  id,
  className,
  description,
}: IArchiveLevelSomViewProps): ReactElement {
  return (
    <ArchiveDescriptionLayout data-testid={dataTestId} id={id} className={className}>
      <EditorPanelSection title={"Sound occlusion"} isFirst>
        <ArchiveDescriptionRow
          label={"Occluders"}
          value={formatCount(description.triangles)}
          caption={describeTwoSided(description)}
        />

        <ArchiveDescriptionRow
          label={"Lets through"}
          value={describeOcclusionRange(description)}
          caption={"The quietest and the loudest face of the mesh, where 1 muffles nothing"}
        />

        <ArchiveDescriptionRow
          label={"Covers"}
          value={formatLevelBounds(description.bounds)}
          caption={"Width, height and depth of what the occluders span"}
        />

        <ArchiveDescriptionRow label={"Version"} value={`${description.version}`} />
      </EditorPanelSection>
    </ArchiveDescriptionLayout>
  );
}
