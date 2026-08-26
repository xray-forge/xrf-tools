import { describe, expect, it } from "@jest/globals";
import { BufferGeometry, Mesh, MeshStandardMaterial, Object3D, Skeleton, SkinnedMesh, Texture } from "three";

import { IVisualPreviewMeshesOptions, VisualPreviewMeshes } from "@/core/visuals/components/scene/VisualPreviewMeshes";
import { IVisualModelViews, IVisualSubmeshViews } from "@/core/visuals/lib/visual-views";
import { mockVisualModelViews } from "@/fixtures/mocks/visual.mocks";
import { Nullable } from "@/lib/types/general";

const CHECKER: Texture = new Texture();

/**
 * One submesh's views, a triangle with two collapse levels so a detail change has somewhere to go.
 *
 * @param index - Submesh index, which is what a texture is addressed by.
 * @param isSkinned - Whether the submesh carries the links that make it drawable by a skinned mesh.
 * @returns Views over arrays built for this test.
 */
function mockSubmesh(index: number, isSkinned: boolean = false): IVisualSubmeshViews {
  return {
    index,
    label: `submesh ${index}`,
    positions: new Float32Array([0, 0, 0, 1, 0, 0, 0, 1, 0]),
    normals: new Float32Array([0, 0, 1, 0, 0, 1, 0, 0, 1]),
    uvs: new Float32Array([0, 0, 1, 0, 0, 1]),
    indices: new Uint16Array([0, 1, 2]),
    skinIndices: isSkinned ? new Uint16Array([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]) : null,
    skinWeights: isSkinned ? new Float32Array([1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0]) : null,
    levels: [
      { start: 0, count: 3, triangleCount: 1 },
      { start: 0, count: 0, triangleCount: 0 },
    ],
  };
}

/**
 * Builds meshes attached to a bare node, which is all they need from a scene.
 *
 * @param model - Model to draw.
 * @param overrides - Options to replace, for the skin or the detail a test is about.
 * @returns The meshes and the node they attached to.
 */
function mockMeshes(
  model: IVisualModelViews = mockVisualModelViews({ submeshes: [mockSubmesh(0)] }),
  overrides: Partial<IVisualPreviewMeshesOptions> = {}
): { meshes: VisualPreviewMeshes; parent: Object3D } {
  const parent: Object3D = new Object3D();

  return {
    parent,
    meshes: VisualPreviewMeshes.create(model, parent, {
      checker: CHECKER,
      detail: 0,
      meshColor: 0xffffff,
      skin: null,
      ...overrides,
    }),
  };
}

/**
 * @param parent - Node the meshes attached to.
 * @returns Every attached mesh, in the order they were added.
 */
function attachedMeshes(parent: Object3D): Array<Mesh<BufferGeometry, MeshStandardMaterial>> {
  return parent.children.filter((it: Object3D): it is Mesh<BufferGeometry, MeshStandardMaterial> => it instanceof Mesh);
}

describe("VisualPreviewMeshes", () => {
  it("attaches a mesh per submesh, named after it", () => {
    const { parent } = mockMeshes(mockVisualModelViews({ submeshes: [mockSubmesh(0), mockSubmesh(7)] }));

    expect(attachedMeshes(parent).map((it) => it.name)).toEqual(["submesh 0", "submesh 7"]);
  });

  it("draws a submesh carrying links as a skinned mesh bound to the skin", () => {
    const skin: Skeleton = new Skeleton([]);
    const { parent } = mockMeshes(mockVisualModelViews({ submeshes: [mockSubmesh(0, true)] }), { skin });

    const mesh: Mesh<BufferGeometry, MeshStandardMaterial> = attachedMeshes(parent)[0];

    expect(mesh).toBeInstanceOf(SkinnedMesh);
    expect((mesh as unknown as SkinnedMesh).skeleton).toBe(skin);
    // Never frustum culled: three.js measures a skinned mesh against its bind pose, and a motion reaches outside that.
    expect(mesh.frustumCulled).toBe(false);
  });

  it("draws a submesh carrying links as plain geometry when the model has no skeleton", () => {
    const { parent } = mockMeshes(mockVisualModelViews({ submeshes: [mockSubmesh(0, true)] }), { skin: null });

    expect(attachedMeshes(parent)[0]).not.toBeInstanceOf(SkinnedMesh);
  });

  it("applies a texture to the submesh it is addressed to", () => {
    const { meshes, parent } = mockMeshes();
    const texture: Texture = new Texture();

    meshes.applyMaterialOptions({ isCheckerVisible: false, isWireframe: false });
    meshes.applyTexture(0, texture);

    expect(attachedMeshes(parent)[0].material.map).toBe(texture);
  });

  it("ignores a texture addressed to a submesh this model does not draw", () => {
    // Borrowed, so there is nothing to clean up: the loader that uploaded it frees it with the rest.
    const { meshes } = mockMeshes();
    const texture: Texture = new Texture();

    let disposals: number = 0;

    texture.addEventListener("dispose", () => {
      disposals += 1;
    });

    meshes.applyTexture(9, texture);

    expect(disposals).toBe(0);
  });

  it("takes the same texture on two submeshes, as a shared file is drawn twice", () => {
    const { meshes, parent } = mockMeshes(mockVisualModelViews({ submeshes: [mockSubmesh(0), mockSubmesh(1)] }));
    const texture: Texture = new Texture();

    meshes.applyTexture(0, texture);
    meshes.applyTexture(1, texture);

    expect(attachedMeshes(parent).map((it) => it.material.map)).toEqual([texture, texture]);
  });

  it("replaces a texture without freeing the one it replaced", () => {
    // One upload is drawn by every submesh naming that file, so freeing on replacement would blank another submesh.
    const { meshes, parent } = mockMeshes();
    const first: Texture = new Texture();
    const second: Texture = new Texture();

    let disposals: number = 0;

    first.addEventListener("dispose", () => {
      disposals += 1;
    });

    meshes.applyTexture(0, first);
    meshes.applyTexture(0, second);

    expect(disposals).toBe(0);
    expect(attachedMeshes(parent)[0].material.map).toBe(second);
  });

  it("stands the checkerboard in for every texture, and gives them back when it is turned off", () => {
    const { meshes, parent } = mockMeshes();
    const texture: Texture = new Texture();

    meshes.applyTexture(0, texture);
    meshes.applyMaterialOptions({ isCheckerVisible: true, isWireframe: false });

    expect(attachedMeshes(parent)[0].material.map).toBe(CHECKER);

    meshes.applyMaterialOptions({ isCheckerVisible: false, isWireframe: false });

    expect(attachedMeshes(parent)[0].material.map).toBe(texture);
  });

  it("keeps a texture that lands while the checkerboard is covering it", () => {
    const { meshes, parent } = mockMeshes();
    const texture: Texture = new Texture();

    meshes.applyMaterialOptions({ isCheckerVisible: true, isWireframe: false });
    meshes.applyTexture(0, texture);

    expect(attachedMeshes(parent)[0].material.map).toBe(CHECKER);

    meshes.applyMaterialOptions({ isCheckerVisible: false, isWireframe: false });

    expect(attachedMeshes(parent)[0].material.map).toBe(texture);
  });

  it("draws wireframe when asked", () => {
    const { meshes, parent } = mockMeshes();

    meshes.applyMaterialOptions({ isCheckerVisible: false, isWireframe: true });

    expect(attachedMeshes(parent)[0].material.wireframe).toBe(true);
  });

  it("moves the draw range down the collapse chain rather than rebuilding geometry", () => {
    const { meshes, parent } = mockMeshes();
    const geometry: BufferGeometry = attachedMeshes(parent)[0].geometry;

    expect(geometry.drawRange).toEqual({ start: 0, count: 3 });

    meshes.setDetailLevel(1);

    expect(attachedMeshes(parent)[0].geometry).toBe(geometry);
    expect(geometry.drawRange).toEqual({ start: 0, count: 0 });
  });

  it("draws at the detail it was built with, so a model opened at reduced detail arrives reduced", () => {
    const { parent } = mockMeshes(undefined, { detail: 1 });

    expect(attachedMeshes(parent)[0].geometry.drawRange).toEqual({ start: 0, count: 0 });
  });

  it("detaches every mesh and frees its geometry and material, leaving the borrowed texture alone", () => {
    const { meshes, parent } = mockMeshes();
    const texture: Texture = new Texture();
    const mesh: Mesh<BufferGeometry, MeshStandardMaterial> = attachedMeshes(parent)[0];

    const freed: Array<string> = [];

    mesh.geometry.addEventListener("dispose", () => freed.push("geometry"));
    mesh.material.addEventListener("dispose", () => freed.push("material"));
    texture.addEventListener("dispose", () => freed.push("texture"));

    meshes.applyTexture(0, texture);
    meshes.dispose();

    expect(parent.children).toHaveLength(0);
    expect(freed).toEqual(["geometry", "material"]);
  });

  it("draws nothing for a model that packed no geometry", () => {
    const { parent }: { parent: Object3D; meshes: Nullable<VisualPreviewMeshes> } = mockMeshes(mockVisualModelViews());

    expect(parent.children).toHaveLength(0);
  });
});
