import { ReactElement } from "react";

import { ArchiveLevelGameDescription, ArchiveLevelGameSpawn } from "@/core/ipc/types/xrf-app";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchiveDescriptionLayout } from "../ArchiveDescriptionLayout";
import { formatCount } from "../ArchiveDescriptionPreview.utils";
import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";
import { describeSpawnKind, describeSpawnProfiles } from "./ArchiveLevelGameView.utils";

interface IArchiveLevelGameViewProps extends BaseComponentProps {
  description: ArchiveLevelGameDescription;
}

/**
 * A level's game data: where a session puts people, and the paths they are given to walk.
 */
export function ArchiveLevelGameView({
  "data-testid": dataTestId = "archive-level-game-view",
  id,
  className,
  description,
}: IArchiveLevelGameViewProps): ReactElement {
  const { spawns } = description;

  return (
    <ArchiveDescriptionLayout data-testid={dataTestId} id={id} className={className}>
      <EditorPanelSection
        title={"Respawn points"}
        caption={"Grouped by what they spawn: a level plants thousands of a few kinds"}
        isFirst
      >
        <ArchiveDescriptionRow
          label={"Points"}
          value={formatCount(description.rpoints)}
          caption={spawns.length ? `Across ${spawns.length} ${spawns.length === 1 ? "kind" : "kinds"}` : null}
        />

        {spawns.map((spawn: ArchiveLevelGameSpawn) => (
          <ArchiveDescriptionRow
            key={spawn.kind}
            label={describeSpawnKind(spawn)}
            value={formatCount(spawn.points)}
            caption={describeSpawnProfiles(spawn)}
          />
        ))}
      </EditorPanelSection>

      <EditorPanelSection title={"Patrol paths"}>
        <ArchiveDescriptionRow
          label={"Paths"}
          value={formatCount(description.ways)}
          caption={`Over ${formatCount(description.wayPoints)} nodes`}
        />

        <ArchiveDescriptionRow
          label={"Without a node"}
          value={formatCount(description.emptyWays)}
          caption={"Paths carrying nothing to walk, which are a path in name only"}
        />
      </EditorPanelSection>
    </ArchiveDescriptionLayout>
  );
}
