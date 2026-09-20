import { describe, expect, it } from "@jest/globals";
import { BufferGeometry, Group, Mesh, MeshBasicMaterial, Object3D, Vector3 } from "three";

import { SUN_MARKER_DISTANCE } from "@/core/level/lib/scene/level-lighting-config";
import { LevelSunMarker } from "@/core/level/lib/scene/LevelSunMarker";

function mockMarker(): { marker: LevelSunMarker; parent: Object3D; mesh: Mesh<BufferGeometry, MeshBasicMaterial> } {
  const parent: Object3D = new Group();
  const marker: LevelSunMarker = new LevelSunMarker(parent);

  return { marker, parent, mesh: parent.children[0] as Mesh<BufferGeometry, MeshBasicMaterial> };
}

describe("LevelSunMarker", () => {
  it("stands in the scene it was given", () => {
    const { parent } = mockMarker();

    expect(parent.children).toHaveLength(1);
  });

  it("stands along the sun's bearing, at the distance it is drawn at", () => {
    const { marker, mesh } = mockMarker();

    marker.setSun(new Vector3(0, 10, 0), 0xffffff);

    expect(mesh.position.toArray()).toEqual([0, SUN_MARKER_DISTANCE, 0]);
  });

  // The light's own position is out past whatever the level spans; the marker is a bearing from the camera, so it
  // stays in the sky of a level of any size.
  it("keeps its bearing from wherever the camera stands", () => {
    const { marker, mesh } = mockMarker();

    marker.setSun(new Vector3(0, 0, 1), 0xffffff);
    marker.follow(new Vector3(100, 20, -50));

    expect(mesh.position.toArray()).toEqual([100, 20, SUN_MARKER_DISTANCE - 50]);
  });

  it("is drawn in the colour of the light it stands for", () => {
    const { marker, mesh } = mockMarker();

    marker.setSun(new Vector3(0, 1, 0), 0xff8800);

    expect(mesh.material.color.getHex()).toBe(0xff8800);
  });

  // A bearing of no length has no direction to take, and normalising it would put the marker in the camera's eye.
  it("keeps the bearing it had when given none", () => {
    const { marker, mesh } = mockMarker();

    marker.setSun(new Vector3(1, 0, 0), 0xffffff);
    marker.setSun(new Vector3(0, 0, 0), 0xffffff);

    expect(mesh.position.toArray()).toEqual([SUN_MARKER_DISTANCE, 0, 0]);
  });

  it("is drawn through the level, like the origin marker", () => {
    const { mesh } = mockMarker();

    expect(mesh.material.depthTest).toBe(false);
    expect(mesh.renderOrder).toBeGreaterThan(0);
  });

  it("goes away when the toolbar asks", () => {
    const { marker, mesh } = mockMarker();

    marker.setVisible(false);

    expect(mesh.visible).toBe(false);
  });

  it("releases what it owns and leaves the scene as it found it", () => {
    const { marker, parent, mesh } = mockMarker();
    const geometry = mesh.geometry;

    let isGeometryDisposed: boolean = false;
    let isMaterialDisposed: boolean = false;

    geometry.addEventListener("dispose", () => (isGeometryDisposed = true));
    mesh.material.addEventListener("dispose", () => (isMaterialDisposed = true));

    marker.dispose();

    expect(parent.children).toHaveLength(0);
    expect(isGeometryDisposed).toBe(true);
    expect(isMaterialDisposed).toBe(true);
  });
});
