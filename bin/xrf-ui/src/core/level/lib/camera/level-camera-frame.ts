import { toDegrees, toDirection, toRadians } from "@xrf/math";
import { Nullable } from "@xrf/types";

import { ILevelCamera } from "@/core/level/lib/camera/level-camera";
import { ILevelGoTo } from "@/core/level/lib/camera/level-camera-goto";
import { IRenderPoint, toXraySpace } from "@/core/render/lib/scene/render-space";

/** How much room a framed sphere leaves round it, of its radius. */
const FRAME_MARGIN: number = 1.25;

/** The nearest a framed sphere is stood off, in metres, so a bottle is seen rather than filled. */
const FRAME_NEAREST: number = 2;

/** Where a camera faces before it has reported: level, a little downward, along `+z`. */
const DEFAULT_FACING: Readonly<Pick<ILevelCamera, "heading" | "pitch">> = { heading: 0, pitch: toRadians(-20) };

/**
 * Unreal's `F`: keeps the camera's rotation and stands it back along its view until a sphere fills the view, with
 * room round it, and no nearer than a couple of metres.
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
  sphere: { center: IRenderPoint; radius: number },
  fieldOfView: number
): ILevelGoTo {
  const { heading, pitch } = camera ?? DEFAULT_FACING;
  const [x, y, z] = toDirection({ heading, pitch });
  const centre: IRenderPoint = toXraySpace(sphere.center);
  const distance: number = Math.max(
    FRAME_NEAREST,
    (sphere.radius * FRAME_MARGIN) / Math.sin(toRadians(fieldOfView) / 2)
  );

  return {
    heading: toDegrees(heading),
    pitch: toDegrees(pitch),
    x: centre.x - x * distance,
    y: centre.y - y * distance,
    z: centre.z - z * distance,
  };
}
