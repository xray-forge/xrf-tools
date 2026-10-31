import { AsyncState } from "@xrf/async";
import { LevelObject, LevelObjectShape, LevelSpawnObjectDetails } from "@xrf/ipc/types/xrf-app-levels";
import { Nullable } from "@xrf/types";
import { EditorPanelProperty, EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Button } from "@xrf/ui/material/button";
import { ReactElement } from "react";

import { formatLevelPoint } from "@/core/level/lib/camera/level-camera";
import { getLevelObjectKind } from "@/core/level/lib/objects/level-object-kinds";
import { toLevelObjectPosition } from "@/core/level/lib/objects/level-object-sphere";
import { describeLevelRestriction, describeLevelShape } from "@/tools/level-viewer/lib/outliner/level-object-describe";
import { describeLevelSpawnRelease } from "@/tools/level-viewer/lib/spawn/level-spawn-release";

import { LevelPlacementProperties } from "./LevelPlacementProperties";

interface ILevelObjectDetailsProps extends BaseComponentProps {
  object: LevelObject;
  /** The visual it is drawn as, by name; empty for one drawn as nothing. */
  visual: string;
  /** What the backend says of a spawned one beyond what the listing holds, read when it was shown. */
  details: AsyncState<LevelSpawnObjectDetails>;
  /** How many other objects are selected with it. */
  others: number;
  /** Where a patrol point leads, as its path's links name the points they reach; empty for anything else. */
  leads: ReadonlyArray<string>;
  onGoTo: () => void;
}

/**
 * What one level object is: its kind and place, what the spawn records of it, what a volume covers, a patrol path's
 * points and links and a point's flags and ways on, what a placement's file or a light's bundle records, and what a
 * spawned one carries.
 */
export function LevelObjectDetails({
  "data-testid": dataTestId = "level-object-details",
  id,
  className,
  object,
  visual,
  details,
  others,
  leads,
  onGoTo,
}: ILevelObjectDetailsProps): ReactElement {
  const read: Nullable<LevelSpawnObjectDetails> = details.value;
  const { patrol, patrolPoint, spawn, volume } = object;

  return (
    <EditorPanelSection
      data-testid={dataTestId}
      id={id}
      className={className}
      title={object.name}
      caption={
        others
          ? `${getLevelObjectKind(object.kind).label}, ${others} more selected`
          : getLevelObjectKind(object.kind).label
      }
    >
      <EditorPanelProperty label={"Position"} value={formatLevelPoint(toLevelObjectPosition(object.transform))} />

      {spawn ? (
        <>
          <EditorPanelProperty label={"Section"} value={spawn.section} isMonospace />
          <EditorPanelProperty label={"Class"} value={spawn.clsid} isMonospace />
          {visual ? <EditorPanelProperty label={"Visual"} value={visual} isMonospace /> : null}
          <EditorPanelProperty label={"Story id"} value={spawn.storyId ?? "None"} />
          <EditorPanelProperty
            label={"New game"}
            value={spawn.release ? describeLevelSpawnRelease(spawn.release) : "Kept"}
          />
        </>
      ) : null}

      {volume ? (
        <>
          <EditorPanelProperty label={"Restricts"} value={describeLevelRestriction(volume.restriction)} />
          {volume.destination ? <EditorPanelProperty label={"Leads to"} value={volume.destination} /> : null}
          <EditorPanelProperty
            label={"Shapes"}
            value={
              volume.shapes.length
                ? volume.shapes.map((shape: LevelObjectShape) => describeLevelShape(shape)).join("\n")
                : "None"
            }
            isMonospace
          />
        </>
      ) : null}

      {patrol ? (
        <>
          <EditorPanelProperty label={"Points"} value={patrol.points.length} />
          <EditorPanelProperty label={"Links"} value={patrol.links.length} />
        </>
      ) : null}

      <LevelPlacementProperties placement={object.placement} light={object.light} />

      {patrolPoint ? (
        <>
          <EditorPanelProperty label={"Flags"} value={patrolPoint.flags} />
          <EditorPanelProperty label={"Level vertex"} value={patrolPoint.levelVertex} />
          <EditorPanelProperty label={"Game vertex"} value={patrolPoint.gameVertex} />
          <EditorPanelProperty label={"Leads to"} value={leads.length ? leads.join("\n") : "Nowhere"} isMonospace />
        </>
      ) : null}

      {spawn ? (
        read ? (
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
        )
      ) : null}

      <Button className={"mt-2"} size={"small"} onClick={onGoTo}>
        {others ? "Frame selection" : "Go to"}
      </Button>
    </EditorPanelSection>
  );
}
