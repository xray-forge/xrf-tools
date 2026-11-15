import { formatCount } from "@xrf/format";
import { ArchiveLevelHomDescription } from "@xrf/ipc/types/xrf-app-archives";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReadingColumn } from "@xrf/ui/layout/reading-column";
import { ReactElement } from "react";

import { formatLevelBounds } from "../ArchiveDescriptionPreview.utils";

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
    <ReadingColumn data-testid={dataTestId} id={id} className={className}>
      <EditorPanelSection title={"Occlusion mesh"}>
        <EditorPanelProperty
          isInline
          label={"Occluders"}
          value={formatCount(description.triangles)}
          caption={"Triangles the renderer culls behind, drawn by nothing itself"}
        />

        <EditorPanelProperty
          isInline
          label={"Covers"}
          value={formatLevelBounds(description.bounds)}
          caption={"Width, height and depth of what the occluders span"}
        />

        <EditorPanelProperty isInline label={"Version"} value={`${description.version}`} />
      </EditorPanelSection>
    </ReadingColumn>
  );
}
