import { describe, expect, it } from "@jest/globals";
import { PerspectiveCamera, Vector3 } from "three";

import { ILevelCamera, toLevelCamera } from "@/core/level/lib/level-camera";

function placed(at: [number, number, number], towards: [number, number, number]): PerspectiveCamera {
  const camera: PerspectiveCamera = new PerspectiveCamera();

  camera.position.set(...at);
  camera.lookAt(new Vector3(...towards));
  camera.updateMatrixWorld();

  return camera;
}

describe("toLevelCamera", () => {
  // The packer negates `z`, so a readout taken straight off the camera would disagree with the level's own data on
  // one axis - and only on one, which is exactly the kind of wrong that goes unnoticed.
  it("states the position in the level's own axes", () => {
    const camera: ILevelCamera = toLevelCamera(placed([10, 20, 30], [10, 20, 0]));

    expect(camera.position).toEqual({ x: 10, y: 20, z: -30 });
  });

  it("reads a heading in the same axes as the position", () => {
    // Looking along renderer -z, which is the level's +z, and so a heading of zero.
    expect(toLevelCamera(placed([0, 0, 0], [0, 0, -1])).heading).toBeCloseTo(0);
    // A quarter turn from it, which the engine measures towards -x.
    expect(toLevelCamera(placed([0, 0, 0], [-1, 0, 0])).heading).toBeCloseTo(Math.PI / 2);
  });

  it("reads pitch positive above the horizon", () => {
    expect(toLevelCamera(placed([0, 0, 0], [0, 1, -1])).pitch).toBeCloseTo(Math.PI / 4);
    expect(toLevelCamera(placed([0, 0, 0], [0, -1, -1])).pitch).toBeCloseTo(-Math.PI / 4);
  });

  // Called on every report for as long as a level is open, which is four times a second for as long as somebody
  // looks at it.
  it("reads into the scratch vector it is given rather than allocating one", () => {
    const facing: Vector3 = new Vector3();

    toLevelCamera(placed([0, 0, 0], [0, 0, -1]), facing);

    expect(facing.z).toBeCloseTo(-1);
  });
});
