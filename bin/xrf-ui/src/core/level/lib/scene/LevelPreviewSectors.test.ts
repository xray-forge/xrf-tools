import { describe, expect, it } from "@jest/globals";
import { Group, InstancedMesh, Mesh, MeshStandardMaterial } from "three";

import { createSectorGeometry } from "@/core/level/lib/sector/level-sector-geometry";
import { ILoadedSector } from "@/core/level/lib/sector/level-sector-set";
import { createSectorViews, ISectorViews } from "@/core/level/lib/sector/level-sector-views";
import { DEFAULT_LEVEL_SURFACE_OPTIONS, getShaderColor } from "@/core/level/lib/surface/level-surface-material";
import {
  mockSectorDescription,
  mockSectorInstanceGroup,
  mockSectorSection,
  mockSectorSurface,
} from "@/fixtures/mocks/level.mocks";
import { MockVisualBuffer } from "@/fixtures/mocks/visual.mocks";

import { LevelPreviewSectors } from "./LevelPreviewSectors";

/** One resident sector drawing the given shader table entries. */
function loadedSector(sector: number, shaderIds: Array<number>): ILoadedSector {
  const buffer: MockVisualBuffer = new MockVisualBuffer();
  const description = mockSectorDescription(buffer, {
    sector,
    sections: shaderIds.map((shaderId: number, index: number) =>
      mockSectorSection({ draw: { count: 3, start: index * 3 }, surface: mockSectorSurface({ shaderId }) })
    ),
  });
  const views: ISectorViews = createSectorViews(
    { ...description, bufferLength: buffer.byteLength },
    buffer.toArrayBuffer()
  );

  return { geometry: createSectorGeometry(views), sector, views };
}

function materialsOf(mesh: Mesh): Array<MeshStandardMaterial> {
  return (Array.isArray(mesh.material) ? mesh.material : [mesh.material]) as Array<MeshStandardMaterial>;
}

describe("LevelPreviewSectors", () => {
  it("adds a mesh for each resident sector", () => {
    const parent: Group = new Group();
    const sectors: LevelPreviewSectors = new LevelPreviewSectors(parent);

    sectors.sync(
      new Map([
        [0, loadedSector(0, [1])],
        [1, loadedSector(1, [2])],
      ])
    );

    expect(sectors.size).toBe(2);
    expect(parent.children).toHaveLength(2);
    expect(parent.children[0]?.name).toBe("sector-0");
  });

  // One material per group, in the order the sections were packed, or the groups past the first draw nothing.
  it("gives each surface of a sector its own material", () => {
    const parent: Group = new Group();
    const sectors: LevelPreviewSectors = new LevelPreviewSectors(parent);

    sectors.sync(new Map([[0, loadedSector(0, [1, 2, 3])]]));

    expect(materialsOf(parent.children[0] as Mesh)).toHaveLength(3);
  });

  // The same surface should read as the same surface wherever it appears, so the colour comes from the entry rather
  // than from the order sectors happened to arrive in.
  it("colours a shader table entry the same way in every sector", () => {
    const parent: Group = new Group();
    const sectors: LevelPreviewSectors = new LevelPreviewSectors(parent);

    sectors.applyViewOptions({ ...DEFAULT_LEVEL_SURFACE_OPTIONS, isSurfaceColored: true });
    sectors.sync(
      new Map([
        [0, loadedSector(0, [7])],
        [1, loadedSector(1, [4, 7])],
      ])
    );

    const first: MeshStandardMaterial = materialsOf(parent.children[0] as Mesh)[0] as MeshStandardMaterial;
    const second: MeshStandardMaterial = materialsOf(parent.children[1] as Mesh)[1] as MeshStandardMaterial;

    expect(first.color.getHex()).toBe(second.color.getHex());
    expect(first.color.getHex()).toBe(getShaderColor(7).getHex());
  });

  it("removes the mesh of a sector that is no longer resident", () => {
    const parent: Group = new Group();
    const sectors: LevelPreviewSectors = new LevelPreviewSectors(parent);
    const resident: Map<number, ILoadedSector> = new Map([[0, loadedSector(0, [1])]]);

    sectors.sync(resident);
    sectors.sync(new Map());

    expect(sectors.size).toBe(0);
    expect(parent.children).toHaveLength(0);
  });

  // A material is device memory like a geometry is, and the loader disposes only the geometry.
  it("disposes the materials of a sector it removes", () => {
    const parent: Group = new Group();
    const sectors: LevelPreviewSectors = new LevelPreviewSectors(parent);

    sectors.sync(new Map([[0, loadedSector(0, [1])]]));

    const material: MeshStandardMaterial = materialsOf(parent.children[0] as Mesh)[0] as MeshStandardMaterial;

    let disposed: boolean = false;

    material.addEventListener("dispose", () => {
      disposed = true;
    });

    sectors.sync(new Map());

    expect(disposed).toBe(true);
  });

  // Syncing runs on every residency change, so rebuilding a mesh that is still resident would throw away and re-upload
  // geometry that never changed.
  it("leaves a sector that is still resident alone", () => {
    const parent: Group = new Group();
    const sectors: LevelPreviewSectors = new LevelPreviewSectors(parent);
    const resident: Map<number, ILoadedSector> = new Map([[0, loadedSector(0, [1])]]);

    sectors.sync(resident);

    const mesh = parent.children[0];

    sectors.sync(resident);

    expect(parent.children[0]).toBe(mesh);
  });

  it("applies the view toggles to every drawn surface", () => {
    const parent: Group = new Group();
    const sectors: LevelPreviewSectors = new LevelPreviewSectors(parent);

    sectors.sync(new Map([[0, loadedSector(0, [1, 2])]]));
    sectors.applyViewOptions({ ...DEFAULT_LEVEL_SURFACE_OPTIONS, isSurfaceColored: false, isWireframe: true });

    for (const material of materialsOf(parent.children[0] as Mesh)) {
      expect(material.wireframe).toBe(true);
      expect(material.color.getHex()).toBe(0xffffff);
    }
  });

  it("releases everything it drew on disposal", () => {
    const parent: Group = new Group();
    const sectors: LevelPreviewSectors = new LevelPreviewSectors(parent);

    sectors.sync(new Map([[0, loadedSector(0, [1])]]));
    sectors.dispose();

    expect(sectors.size).toBe(0);
    expect(parent.children).toHaveLength(0);
  });
});

describe("LevelPreviewSectors instances", () => {
  /** One resident sector standing a mesh in the given places. */
  function loadedInstances(sector: number, places: Array<number>): ILoadedSector {
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const description = mockSectorDescription(buffer, {
      instances: [mockSectorInstanceGroup(buffer, places)],
      sector,
    });
    const views: ISectorViews = createSectorViews(
      { ...description, bufferLength: buffer.byteLength },
      buffer.toArrayBuffer()
    );

    return { geometry: createSectorGeometry(views), sector, views };
  }

  it("draws an instanced mesh beside the sector's own geometry", () => {
    const parent: Group = new Group();
    const sectors: LevelPreviewSectors = new LevelPreviewSectors(parent);

    sectors.sync(new Map([[0, loadedInstances(0, [0, 100])]]));

    const instanced = parent.children.filter((child): child is InstancedMesh => child instanceof InstancedMesh);

    expect(instanced).toHaveLength(1);
    expect(instanced[0]?.count).toBe(2);
  });

  // A sector of a swamp bakes nothing in place: every drawable it reaches is a tree the level stands. Adding a mesh
  // over its empty index array would be a draw call to say nothing.
  it("adds no baked mesh for a sector the level bakes nothing of", () => {
    const parent: Group = new Group();
    const sectors: LevelPreviewSectors = new LevelPreviewSectors(parent);
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const description = mockSectorDescription(buffer, {
      instances: [mockSectorInstanceGroup(buffer, [0, 100])],
      sections: [],
    });
    const views: ISectorViews = createSectorViews(
      { ...description, bufferLength: buffer.byteLength },
      buffer.toArrayBuffer()
    );

    sectors.sync(new Map([[0, { geometry: createSectorGeometry(views), sector: 0, views }]]));

    expect(parent.children).toHaveLength(1);
    expect(parent.children[0]).toBeInstanceOf(InstancedMesh);
  });

  // The loader owns the sector's geometry, but an instanced mesh built its own here: leaving it behind leaks one
  // upload per stand of trees every time the camera moves on.
  it("disposes the geometry it built for an instanced mesh", () => {
    const parent: Group = new Group();
    const sectors: LevelPreviewSectors = new LevelPreviewSectors(parent);

    sectors.sync(new Map([[0, loadedInstances(0, [0])]]));

    const instanced = parent.children.find((child): child is InstancedMesh => child instanceof InstancedMesh);

    let disposed: boolean = false;

    instanced?.geometry.addEventListener("dispose", () => {
      disposed = true;
    });

    sectors.sync(new Map());

    expect(disposed).toBe(true);
    expect(parent.children).toHaveLength(0);
  });
});
