import { IPoint3, toDegrees, toDirection, toRadians, toSphereFitDistance, toXraySpace } from "@xrf/math";
import { Nullable } from "@xrf/types";

import { ILevelCamera } from "@/core/level/lib/camera/level-camera";
import { ILevelGoTo } from "@/core/level/lib/camera/level-camera-goto";

/** How much room a framed sphere leaves round it, of its radius. */
const FRAME_MARGIN: number = 1.25;

/** The nearest a framed sphere is stood off, in metres, so a bottle is seen rather than filled. */
const FRAME_NEAREST: number = 2;

/**
 * The least a framing camera looks down, so it stands above what it frames: a path or a point lying on the ground,
 * framed level, would stand the camera at the ground's height and inside the terrain rising behind it.
 */
const FRAME_LEAST_DOWNWARD: number = toRadians(-20);

/** Where a camera faces before it has reported: along `+z`, looking down as little as framing does. */
const DEFAULT_FACING: Readonly<Pick<ILevelCamera, "heading" | "pitch">> = {
  heading: 0,
  pitch: FRAME_LEAST_DOWNWARD,
};

/**
 * Unreal's `F`: keeps the camera's heading and stands it back along its view until a sphere fills the view, with
 * room round it, and no nearer than a couple of metres; it keeps its pitch where that looks down steeply enough and
 * looks down that much where it does not, so the camera stands above what it frames.
 *
 * @param camera - Where the camera stands and faces, or null before it has reported.
 * @param sphere - What to frame, in renderer space.
 * @param sphere.center - Its centre.
 * @param sphere.radius - Its radius.
 * @param fieldOfView - The camera's vertical field of view, in degrees.
 * @returns Where to go, as the readout states it.
 */
export function toLevelFramedGoTo(
  camera: Nullable<ILevelCamera>,
  sphere: { center: IPoint3; radius: number },
  fieldOfView: number
): ILevelGoTo {
  const facing: Pick<ILevelCamera, "heading" | "pitch"> = camera ?? DEFAULT_FACING;
  const heading: number = facing.heading;
  const pitch: number = Math.min(facing.pitch, FRAME_LEAST_DOWNWARD);
  const [x, y, z] = toDirection({ heading, pitch });
  const centre: IPoint3 = toXraySpace(sphere.center);
  const distance: number = Math.max(
    FRAME_NEAREST,
    toSphereFitDistance(sphere.radius, toRadians(fieldOfView), FRAME_MARGIN)
  );

  return {
    heading: toDegrees(heading),
    pitch: toDegrees(pitch),
    x: centre.x - x * distance,
    y: centre.y - y * distance,
    z: centre.z - z * distance,
  };
}
