import { ELevelObjectShape, LevelObject, LevelObjectShape } from "@xrf/ipc/types/xrf-app-levels";
import { Vector3d } from "@xrf/ipc/types/xrf-math";
import { VisualTransform } from "@xrf/ipc/types/xrf-visual";
import { IPoint3, toPoint3, toXraySpace } from "@xrf/math";
import { Nullable } from "@xrf/types";

/** What an object is taken to span with nothing else to measure: a metre about where it stands. */
const UNREAD_RADIUS: number = 1;

/** A sphere in renderer space. */
export interface ILevelObjectSphere {
  center: IPoint3;
  radius: number;
}

/**
 * @param transform - Where an object stands, in renderer space.
 * @param point - A point of its own space.
 * @returns The point where the transform stands it.
 */
function toWorldPoint(transform: VisualTransform, point: IPoint3): IPoint3 {
  const [c, i, j, k] = [transform.c, transform.i, transform.j, transform.k].map(toPoint3) as [
    IPoint3,
    IPoint3,
    IPoint3,
    IPoint3,
  ];

  return {
    x: c.x + i.x * point.x + j.x * point.y + k.x * point.z,
    y: c.y + i.y * point.x + j.y * point.y + k.y * point.z,
    z: c.z + i.z * point.x + j.z * point.y + k.z * point.z,
  };
}

/**
 * `Fbox::getsphere` of the box holding every sphere: its middle, and the distance to its far corner.
 *
 * @param spheres - What is held, a point a sphere of no radius.
 * @returns The sphere about the box; null for nothing held.
 */
function toSphereAboutBox(spheres: ReadonlyArray<ILevelObjectSphere>): Nullable<ILevelObjectSphere> {
  if (!spheres.length) {
    return null;
  }

  const axes: Array<"x" | "y" | "z"> = ["x", "y", "z"];
  const [low, high] = [-1, 1].map((side: number) =>
    axes.map((axis: "x" | "y" | "z") =>
      side < 0
        ? Math.min(...spheres.map((sphere: ILevelObjectSphere) => sphere.center[axis] - sphere.radius))
        : Math.max(...spheres.map((sphere: ILevelObjectSphere) => sphere.center[axis] + sphere.radius))
    )
  ) as [Array<number>, Array<number>];
  const [x, y, z] = axes.map((_, axis: number) => ((low[axis] as number) + (high[axis] as number)) / 2) as [
    number,
    number,
    number,
  ];

  return {
    center: { x, y, z },
    radius: Math.hypot((high[0] as number) - x, (high[1] as number) - y, (high[2] as number) - z),
  };
}

/**
 * @param transform - Where a volume stands, in renderer space.
 * @param shape - One of its shapes, in its own space.
 * @returns What the shape holds where it stands: a sphere as it is, a box by its corners.
 */
function toShapeSpheres(transform: VisualTransform, shape: LevelObjectShape): Array<ILevelObjectSphere> {
  if (shape.kind === ELevelObjectShape.SPHERE) {
    return [{ center: toWorldPoint(transform, toPoint3(shape.center)), radius: shape.radius ?? 0 }];
  }

  const [i, j, k] = shape.axes.map(toPoint3) as [IPoint3, IPoint3, IPoint3];
  const center: IPoint3 = toPoint3(shape.center);

  // The unit cube's corners where the box's axes and the object's transform stand them.
  return Array.from({ length: 8 }, (_, corner: number) => {
    const [x, y, z] = [corner & 1, (corner >> 1) & 1, (corner >> 2) & 1].map((bit: number) => bit - 0.5) as [
      number,
      number,
      number,
    ];

    return {
      center: toWorldPoint(transform, {
        x: center.x + i.x * x + j.x * y + k.x * z,
        y: center.y + i.y * x + j.y * y + k.y * z,
        z: center.z + i.z * x + j.z * y + k.z * z,
      }),
      radius: 0,
    };
  });
}

/**
 * @param object - A level object.
 * @param drawn - The sphere its model spans where it stands, in renderer space, centre then radius, as the renderer
 *   holds it; null for an object not drawn yet, or drawn as nothing.
 * @returns What it spans: its model, else the box about its volume's shapes as `CCF_Shape::ComputeBounds` measures it,
 *   else the box about a patrol path's points, else a metre about where it stands.
 */
export function toLevelObjectSphere(
  object: LevelObject,
  drawn: Nullable<readonly [number, number, number, number]>
): ILevelObjectSphere {
  if (drawn) {
    return { center: { x: drawn[0], y: drawn[1], z: drawn[2] }, radius: drawn[3] || UNREAD_RADIUS };
  }

  const spanned: Array<ILevelObjectSphere> = object.volume
    ? object.volume.shapes.flatMap((shape: LevelObjectShape) => toShapeSpheres(object.transform, shape))
    : (object.patrol?.points ?? []).map((point: Vector3d) => ({ center: toPoint3(point), radius: 0 }));
  const sphere: Nullable<ILevelObjectSphere> = toSphereAboutBox(spanned);

  return sphere && sphere.radius > 0 ? sphere : { center: toPoint3(object.transform.c), radius: UNREAD_RADIUS };
}

/**
 * @param spheres - What several objects span.
 * @returns A sphere holding them all, about the box holding them; null for none.
 */
export function toLevelObjectsSphere(spheres: ReadonlyArray<ILevelObjectSphere>): Nullable<ILevelObjectSphere> {
  return spheres.length === 1 ? (spheres[0] as ILevelObjectSphere) : toSphereAboutBox(spheres);
}

/**
 * @param transform - Where an object stands, in renderer space.
 * @returns Where, as the engine states it.
 */
export function toLevelObjectPosition(transform: VisualTransform): IPoint3 {
  return toXraySpace(toPoint3(transform.c));
}
