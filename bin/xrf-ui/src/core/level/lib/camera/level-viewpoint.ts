import { Maybe, Nullable } from "@xrf/types";

import { LevelStart } from "@/core/ipc/types/xrf-app";
import { Vector3d } from "@/core/ipc/types/xrf-math";
import { VisualBounds } from "@/core/ipc/types/xrf-visual";
import { ILevelPoint } from "@/core/level/lib/residency/level-residency";

/** Where the camera stands and what it looks at, both in renderer space. */
export interface ILevelViewpoint {
  position: ILevelPoint;
  target: ILevelPoint;
}

/** Where a level with no usable extent opens, which is the only place that is certainly meaningful. */
const ORIGIN: ILevelPoint = { x: 0, y: 0, z: 0 };

/** How far along the axis it faces the camera looks, as a fraction of that axis; anything positive aims it level. */
const LOOK_AHEAD: number = 0.25;

/** The shortest look the camera is given, so a level with no extent still gets a direction rather than a point. */
const MINIMUM_LOOK: number = 1;

/**
 * Where a level opens: where the backend says the game puts a player on it, or else standing in the middle of it,
 * looking down its longer side.
 *
 * @param bounds - The level's extent, as the backend measured it, or null for a level that reports none.
 * @param start - Where the backend opens the level, or null where it names nowhere.
 * @returns Where to stand and what to look at.
 */
export function toLevelStartViewpoint(
  bounds: Maybe<VisualBounds>,
  start: Nullable<LevelStart> = null
): ILevelViewpoint {
  if (start) {
    const position: ILevelPoint = toPoint(start.position);
    const direction: ILevelPoint = toPoint(start.direction);

    return {
      position,
      target: { x: position.x + direction.x, y: position.y + direction.y, z: position.z + direction.z },
    };
  }

  const minimum: ILevelPoint = toPoint(bounds?.boundingBox.min);
  const maximum: ILevelPoint = toPoint(bounds?.boundingBox.max);

  const position: ILevelPoint = {
    x: (minimum.x + maximum.x) / 2,
    y: (minimum.y + maximum.y) / 2,
    z: (minimum.z + maximum.z) / 2,
  };

  const across: number = Math.abs(maximum.x - minimum.x);
  const along: number = Math.abs(maximum.z - minimum.z);
  // Down the longer side, so the most of the level is in front of the camera rather than behind it. Level with the
  // horizon either way: a camera that opens looking at the ground shows a texture, not a place. Never zero, because a
  // camera told to look at where it stands has nothing to turn towards and keeps whatever it was left pointing at.
  const ahead: number = Math.max(Math.max(across, along) * LOOK_AHEAD, MINIMUM_LOOK);

  return {
    position,
    target: across >= along ? { ...position, x: position.x + ahead } : { ...position, z: position.z - ahead },
  };
}

/** A point from the wire, where every field is optional because a float may not be one. */
function toPoint(corner: Maybe<Vector3d>): ILevelPoint {
  return {
    x: corner?.x ?? ORIGIN.x,
    y: corner?.y ?? ORIGIN.y,
    z: corner?.z ?? ORIGIN.z,
  };
}
