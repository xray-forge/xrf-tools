import { formatBytes, formatCount } from "@xrf/format";
import { ArchiveLevelCollisionDescription } from "@xrf/ipc/types/xrf-app-archives";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReadingColumn } from "@xrf/ui/layout/reading-column";
import { ReactElement } from "react";

import { formatLevelBounds } from "../ArchiveDescriptionPreview.utils";

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
    <ReadingColumn data-testid={dataTestId} id={id} className={className}>
      <EditorPanelSection title={"Collision mesh"}>
        <EditorPanelProperty
          isInline
          label={"Faces"}
          value={formatCount(description.faces)}
          caption={`Over ${formatCount(description.vertices)} vertices`}
        />

        <EditorPanelProperty
          isInline
          label={"Covers"}
          value={formatLevelBounds(description.bounds)}
          caption={"Width, height and depth of the box it declares"}
        />

        <EditorPanelProperty isInline label={"Version"} value={`${description.version}`} />

        <EditorPanelProperty
          isInline
          label={"Size"}
          value={formatBytes(description.size)}
          caption={"The mesh itself is read past; only the header is read"}
        />
      </EditorPanelSection>
    </ReadingColumn>
  );
}
