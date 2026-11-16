import { formatCount } from "@xrf/format";
import { ArchiveLevelSomDescription } from "@xrf/ipc/types/xrf-app-archives";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReadingColumn } from "@xrf/ui/layout/reading-column";
import { ReactElement } from "react";

import { formatLevelBounds } from "../ArchiveDescriptionPreview.utils";
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
    <ReadingColumn data-testid={dataTestId} id={id} className={className}>
      <EditorPanelSection title={"Sound occlusion"}>
        <EditorPanelProperty
          isInline
          label={"Occluders"}
          value={formatCount(description.triangles)}
          caption={describeTwoSided(description)}
        />

        <EditorPanelProperty
          isInline
          label={"Lets through"}
          value={describeOcclusionRange(description)}
          caption={"The quietest and the loudest face of the mesh, where 1 muffles nothing"}
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
