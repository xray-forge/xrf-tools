import { AsyncState } from "@xrf/async";
import { LevelSpawnObject, LevelSpawnObjectDetails } from "@xrf/ipc/types/xrf-app-levels";
import { Nullable } from "@xrf/types";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Button } from "@xrf/ui/material/button";
import { ReactElement } from "react";

import { formatLevelPoint } from "@/core/level/lib/camera/level-camera";
import { toLevelSpawnPosition } from "@/core/level/lib/spawn/level-spawn-sphere";
import { describeLevelSpawnRelease } from "@/tools/level-viewer/lib/spawn/level-spawn-release";

interface ILevelSpawnDetailsProps extends BaseComponentProps {
  object: LevelSpawnObject;
  /** The visual it stands as, by name. */
  visual: string;
  /** What the backend says of it beyond what the tree holds, read when it was chosen. */
  details: AsyncState<LevelSpawnObjectDetails>;
  onGoTo: () => void;
}

/**
 * What one spawned object is, under the tree it was chosen in: what the game spawns it as, where, and what it carries.
 */
export function LevelSpawnDetails({
  "data-testid": dataTestId = "level-spawn-details",
  id,
  className,
  object,
  visual,
  details,
  onGoTo,
}: ILevelSpawnDetailsProps): ReactElement {
  const read: Nullable<LevelSpawnObjectDetails> = details.value;

  return (
    <EditorPanelSection
      data-testid={dataTestId}
      id={id}
      className={className}
      title={object.name}
      caption={object.section}
    >
      <EditorPanelProperty label={"Class"} value={object.clsid} isMonospace />
      <EditorPanelProperty label={"Visual"} value={visual} isMonospace />
      <EditorPanelProperty label={"Position"} value={formatLevelPoint(toLevelSpawnPosition(object.transform))} />
      <EditorPanelProperty label={"Story id"} value={object.storyId ?? "None"} />
      <EditorPanelProperty
        label={"New game"}
        value={object.release ? describeLevelSpawnRelease(object.release) : "Kept"}
      />

      {read ? (
        <>
          <EditorPanelProperty label={"ALife id"} value={read.id} />
          <EditorPanelProperty label={"Game vertex"} value={read.gameVertexId ?? "None"} />
          <EditorPanelProperty label={"Level vertex"} value={read.levelVertexId ?? "None"} />
          <EditorPanelProperty label={"Custom data"} value={read.customData.trim() || "None"} isMonospace />
        </>
      ) : (
        <EditorPanelProperty
          label={"Custom data"}
          value={details.isFailed ? (details.error?.message ?? "Unreadable") : "Reading"}
        />
      )}

      <Button className={"mt-2"} size={"small"} onClick={onGoTo}>
        Go to
      </Button>
    </EditorPanelSection>
  );
}
