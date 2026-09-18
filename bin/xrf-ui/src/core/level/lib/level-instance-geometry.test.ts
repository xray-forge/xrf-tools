import { describe, expect, it } from "@jest/globals";
import { BufferGeometry, InstancedMesh, Matrix4, MeshStandardMaterial, Vector3 } from "three";

import { SectorDescription } from "@/core/ipc/types/xrf-visual";
import { createInstancedMesh } from "@/core/level/lib/level-instance-geometry";
import { createGeometry } from "@/core/level/lib/level-sector-geometry";
import { createSectorViews, ISectorInstanceViews, ISectorViews } from "@/core/level/lib/level-sector-views";
import { mockSectorDescription, mockSectorInstanceGroup } from "@/fixtures/mocks/level.mocks";
import { MockVisualBuffer } from "@/fixtures/mocks/visual.mocks";

/** A sector standing one mesh in the given places along x. */
function instancesAt(places: Array<number>): ISectorInstanceViews {
  const buffer: MockVisualBuffer = new MockVisualBuffer();
  const description: SectorDescription = mockSectorDescription(buffer, {
    instances: [mockSectorInstanceGroup(buffer, places)],
  });
  const views: ISectorViews = createSectorViews(
    { ...description, bufferLength: buffer.byteLength },
    buffer.toArrayBuffer()
  );

  return views.instances[0] as ISectorInstanceViews;
}

describe("level instance geometry", () => {
  it("builds one geometry for a mesh however many places it stands", () => {
    const group: ISectorInstanceViews = instancesAt([0, 100, 200]);
    const geometry: BufferGeometry = createGeometry(group.geometry);

    expect(geometry.getAttribute("position").count).toBe(3);
    expect(geometry.getIndex()?.count).toBe(3);
    expect(group.instanceCount).toBe(3);
  });

  it("stands the mesh in every place the level puts it", () => {
    const group: ISectorInstanceViews = instancesAt([0, 100, -50]);
    const mesh: InstancedMesh = createInstancedMesh(group, createGeometry(group.geometry), new MeshStandardMaterial());

    expect(mesh.count).toBe(3);

    const matrix: Matrix4 = new Matrix4();
    const placed: Array<number> = [];

    for (let index = 0; index < mesh.count; index += 1) {
      mesh.getMatrixAt(index, matrix);
      placed.push(new Vector3().setFromMatrixPosition(matrix).x);
    }

    expect(placed).toEqual([0, 100, -50]);
  });

  // The engine stores a row-vector matrix row major and three.js reads a column-vector one column major. Those are
  // transposes, so the same floats mean the same transform - reading them the other way puts every tree of a level
  // somewhere else entirely.
  it("reads the engine's matrix without rearranging it", () => {
    const group: ISectorInstanceViews = instancesAt([42]);
    const mesh: InstancedMesh = createInstancedMesh(group, createGeometry(group.geometry), new MeshStandardMaterial());
    const matrix: Matrix4 = new Matrix4();

    mesh.getMatrixAt(0, matrix);

    const placed: Vector3 = new Vector3(0, 0, 0).applyMatrix4(matrix);

    expect(placed.x).toBe(42);
    expect(placed.y).toBe(0);
    expect(placed.z).toBe(0);
  });

  // Instances are placed by their own matrices rather than by the tree's world position, so a frustum test against an
  // unmoved bounding sphere would cull the whole stand the moment the origin left the view.
  it("leaves culling to the viewer rather than to an unmoved bounding sphere", () => {
    const group: ISectorInstanceViews = instancesAt([1000]);
    const mesh: InstancedMesh = createInstancedMesh(group, createGeometry(group.geometry), new MeshStandardMaterial());

    expect(mesh.frustumCulled).toBe(false);
  });
});
