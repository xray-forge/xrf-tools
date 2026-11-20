import { formatCount } from "@xrf/format";
import { ArchiveLevelGameDescription, ArchiveLevelGameSpawn } from "@xrf/ipc/types/xrf-app-archives";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReadingColumn } from "@xrf/ui/layout/reading-column";
import { ReactElement } from "react";

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
    <ReadingColumn data-testid={dataTestId} id={id} className={className}>
      <EditorPanelSection
        title={"Respawn points"}
        caption={"Grouped by what they spawn: a level plants thousands of a few kinds"}
      >
        <EditorPanelProperty
          isInline
          label={"Points"}
          value={formatCount(description.rpoints)}
          caption={spawns.length ? `Across ${spawns.length} ${spawns.length === 1 ? "kind" : "kinds"}` : null}
        />

        {spawns.map((spawn: ArchiveLevelGameSpawn) => (
          <EditorPanelProperty
            isInline
            key={spawn.kind}
            label={describeSpawnKind(spawn)}
            value={formatCount(spawn.points)}
            caption={describeSpawnProfiles(spawn)}
          />
        ))}
      </EditorPanelSection>

      <EditorPanelSection title={"Patrol paths"}>
        <EditorPanelProperty
          isInline
          label={"Paths"}
          value={formatCount(description.ways)}
          caption={`Over ${formatCount(description.wayPoints)} nodes`}
        />

        <EditorPanelProperty
          isInline
          label={"Without a node"}
          value={formatCount(description.emptyWays)}
          caption={"Paths carrying nothing to walk, which are a path in name only"}
        />
      </EditorPanelSection>
    </ReadingColumn>
  );
}
