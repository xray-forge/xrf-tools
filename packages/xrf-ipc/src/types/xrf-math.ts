// Auto-generated rust bindings. Do not edit it manually.

/** An axis-aligned box, the engine's `Fbox`: its lowest and highest corner. */
export type Aabb = {
  min: Vector3d;
  max: Vector3d;
};

/**
 * An `Fmatrix` stored without its fourth column, as `NET_Packet::w_matrix` writes one: the three axis rows `i`, `j`
 * and `k`, then the translation `c`. Reading one back fixes the dropped column at `0, 0, 0, 1`, so what it holds is
 * an affine transform.
 */
export type Affine3d = {
  /** `i`, where the transform takes the x axis. */
  xAxis: Vector3d;
  /** `j`. */
  yAxis: Vector3d;
  /** `k`. */
  zAxis: Vector3d;
  /** `c`. */
  translation: Vector3d;
};

/** A sphere, the engine's `Fsphere`: its centre, `P`, and radius, `R`. */
export type Sphere = {
  center: Vector3d;
  radius: number | null;
};

export type Vector3d<T = number | null> = {
  x: T;
  y: T;
  z: T;
};
