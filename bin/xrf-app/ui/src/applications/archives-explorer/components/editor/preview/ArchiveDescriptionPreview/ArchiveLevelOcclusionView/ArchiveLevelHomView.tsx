import { formatCount } from "@xrf/format";
import { ArchiveLevelHomDescription } from "@xrf/ipc/types/xrf-app-archives";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { EditorPanelSection } from "@xrf/ui/shell/editor/EditorPanel/EditorPanelSection";
import { ReactElement } from "react";

import { ArchiveDescriptionLayout } from "../ArchiveDescriptionLayout";
import { formatLevelBounds } from "../ArchiveDescriptionPreview.utils";
import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";

interface IArchiveLevelHomViewProps extends BaseComponentProps {
  description: ArchiveLevelHomDescription;
}

/**
 * A level's occlusion mesh: the geometry the renderer hides the world behind.
 */
export function ArchiveLevelHomView({
  "data-testid": dataTestId = "archive-level-hom-view",
  id,
  className,
  description,
}: IArchiveLevelHomViewProps): ReactElement {
  return (
    <ArchiveDescriptionLayout data-testid={dataTestId} id={id} className={className}>
      <EditorPanelSection title={"Occlusion mesh"} isFirst>
        <ArchiveDescriptionRow
          label={"Occluders"}
          value={formatCount(description.triangles)}
          caption={"Triangles the renderer culls behind, drawn by nothing itself"}
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
