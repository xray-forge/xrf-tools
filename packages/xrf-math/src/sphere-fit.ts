/**
 * How far back a camera stands for a sphere to fill its view.
 *
 * @param radius - The sphere's radius.
 * @param fieldOfView - The camera's field of view, in radians.
 * @param margin - How much room to leave around the sphere, one for none.
 * @returns The distance from the sphere's centre.
 */
export function toSphereFitDistance(radius: number, fieldOfView: number, margin: number): number {
  return (radius * margin) / Math.sin(fieldOfView / 2);
}
