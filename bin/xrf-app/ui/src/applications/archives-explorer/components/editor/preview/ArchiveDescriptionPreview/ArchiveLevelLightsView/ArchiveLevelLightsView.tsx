import { formatCount } from "@xrf/format";
import { ArchiveLevelLightsDescription, ArchiveLevelLightsGroup } from "@xrf/ipc/types/xrf-app-archives";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";

import { ArchiveDescriptionLayout } from "../ArchiveDescriptionLayout";
import { formatLevelBounds } from "../ArchiveDescriptionPreview.utils";
import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";
import { describeGroupContents, describeGroupUse } from "./ArchiveLevelLightsView.utils";

interface IArchiveLevelLightsViewProps extends BaseComponentProps {
  description: ArchiveLevelLightsDescription;
}

/**
 * A level's compiled lights: what the compiler left behind, and the part of it the runtime takes.
 */
export function ArchiveLevelLightsView({
  "data-testid": dataTestId = "archive-level-lights-view",
  id,
  className,
  description,
}: IArchiveLevelLightsViewProps): ReactElement {
  const { groups } = description;

  return (
    <ArchiveDescriptionLayout data-testid={dataTestId} id={id} className={className}>
      <EditorPanelSection title={"Compiled lights"} isFirst>
        <ArchiveDescriptionRow
          label={"Lights"}
          value={formatCount(description.lights)}
          caption={"Everything the compiler wrote, across every chunk of the file"}
        />

        <ArchiveDescriptionRow
          label={"Reaching the runtime"}
          value={formatCount(description.used)}
          caption={"The point lights of the header chunk, which is the only one CLight_DB::LoadHemi opens"}
        />

        <ArchiveDescriptionRow
          label={"Covers"}
          value={formatLevelBounds(description.bounds)}
          caption={"Width, height and depth of where the lights stand"}
        />
      </EditorPanelSection>

      <EditorPanelSection
        title={`Chunks (${groups.length})`}
        caption={"In the order the compiler wrote them, each under the id it wrote it as"}
      >
        {groups.map((group: ArchiveLevelLightsGroup) => (
          <ArchiveDescriptionRow
            key={group.id}
            label={`Chunk ${group.id}`}
            value={describeGroupContents(group)}
            caption={describeGroupUse(group)}
          />
        ))}
      </EditorPanelSection>
    </ArchiveDescriptionLayout>
  );
}
