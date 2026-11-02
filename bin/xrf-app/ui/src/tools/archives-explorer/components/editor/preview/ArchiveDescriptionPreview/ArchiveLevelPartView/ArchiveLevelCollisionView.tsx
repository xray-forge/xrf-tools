import { formatBytes, formatCount } from "@xrf/format";
import { ArchiveLevelCollisionDescription } from "@xrf/ipc/types/xrf-app-archives";
import { EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { ArchiveDescriptionLayout } from "../ArchiveDescriptionLayout";
import { formatLevelBounds } from "../ArchiveDescriptionPreview.utils";
import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";

interface IArchiveLevelCollisionViewProps extends BaseComponentProps {
  description: ArchiveLevelCollisionDescription;
}

/**
 * A level's collision mesh, read from the header alone.
 */
export function ArchiveLevelCollisionView({
  "data-testid": dataTestId = "archive-level-collision-view",
  id,
  className,
  description,
}: IArchiveLevelCollisionViewProps): ReactElement {
  return (
    <ArchiveDescriptionLayout data-testid={dataTestId} id={id} className={className}>
      <EditorPanelSection title={"Collision mesh"} isFirst>
        <ArchiveDescriptionRow
          label={"Faces"}
          value={formatCount(description.faces)}
          caption={`Over ${formatCount(description.vertices)} vertices`}
        />

        <ArchiveDescriptionRow
          label={"Covers"}
          value={formatLevelBounds(description.bounds)}
          caption={"Width, height and depth of the box it declares"}
        />

        <ArchiveDescriptionRow label={"Version"} value={`${description.version}`} />

        <ArchiveDescriptionRow
          label={"Size"}
          value={formatBytes(description.size)}
          caption={"The mesh itself is read past; only the header is read"}
        />
      </EditorPanelSection>
    </ArchiveDescriptionLayout>
  );
}
