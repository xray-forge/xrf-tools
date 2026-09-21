import { describe, expect, it } from "@jest/globals";
import { MeshStandardMaterial, Texture } from "three";

import { LevelMaterialSet } from "@/core/level/lib/surface/level-material-set";
import { DEFAULT_LEVEL_SURFACE_OPTIONS, ILevelSurface } from "@/core/level/lib/surface/level-surface-material";
import { OPAQUE_RENDER_SURFACE, toRenderSurface } from "@/core/render/lib/surface/render-surface";
import { IMockLevelTextureSource, mockLevelTextureSource, mockSectorSurface } from "@/fixtures/mocks/level.mocks";
import { mockAlphaSurfaceDescriptor } from "@/fixtures/mocks/visual.mocks";

/** A surface of one shader table entry, as two different sectors would each name it. */
function surfaceOf(shaderId: number, overrides: Partial<ILevelSurface> = {}): ILevelSurface {
  return {
    render: OPAQUE_RENDER_SURFACE,
    surface: mockSectorSurface({ shaderId }),
    ...overrides,
  };
}

function lookup(...references: Array<string>): IMockLevelTextureSource {
  return mockLevelTextureSource(
    Object.fromEntries(
      references.map((reference: string) => [
        reference,
        { isAlphaRead: false, isMipped: true, reason: null, texture: new Texture(), upload: null },
      ])
    )
  );
}

describe("LevelMaterialSet", () => {
  // One ground shader dresses dozens of sectors. A material each meant marsh held 857 for 501 distinct surfaces,
  // every one of them re-dressed on each view toggle.
  it("gives two sectors naming one shader table entry the same material", () => {
    const set: LevelMaterialSet = new LevelMaterialSet();

    const first: MeshStandardMaterial = set.claim(surfaceOf(7));
    const second: MeshStandardMaterial = set.claim(surfaceOf(7));

    expect(second).toBe(first);
    expect(set.size).toBe(1);
  });

  // One row, one material, whatever geometry names it: what a surface is drawn with is the row's answer, and
  // nothing about a sector's attributes changes it now that no attribute is read as light.
  it("shares one material between two sectors naming one row", () => {
    const set: LevelMaterialSet = new LevelMaterialSet();

    set.claim(surfaceOf(7));
    set.claim(surfaceOf(7));

    expect(set.size).toBe(1);
  });

  it("tells two shader table entries apart", () => {
    const set: LevelMaterialSet = new LevelMaterialSet();

    set.claim(surfaceOf(1));
    set.claim(surfaceOf(2));

    expect(set.size).toBe(2);
  });

  it("keeps what the sectors on screen still name and disposes the rest", () => {
    const set: LevelMaterialSet = new LevelMaterialSet();
    const kept: MeshStandardMaterial = set.claim(surfaceOf(1));
    const dropped: MeshStandardMaterial = set.claim(surfaceOf(2));

    let disposed: boolean = false;

    dropped.addEventListener("dispose", () => {
      disposed = true;
    });

    set.retain([surfaceOf(1)]);

    expect(set.size).toBe(1);
    expect(disposed).toBe(true);
    // The one still named is the same object, not a rebuild of it.
    expect(set.claim(surfaceOf(1))).toBe(kept);
  });

  it("dresses what it already holds when the view state changes", () => {
    const set: LevelMaterialSet = new LevelMaterialSet();
    const material: MeshStandardMaterial = set.claim(
      surfaceOf(7, { render: toRenderSurface(mockAlphaSurfaceDescriptor()) })
    );

    expect(material.wireframe).toBe(false);

    set.applyViewOptions({ ...DEFAULT_LEVEL_SURFACE_OPTIONS, isWireframe: true });

    expect(material.wireframe).toBe(true);
    // What its blender says is not a view state: a surface that cuts out keeps cutting out through every toggle.
    expect(material.alphaTest).toBeCloseTo(200 / 255);
  });

  it("dresses what it already holds when the textures arrive", () => {
    // Materials are claimed as a sector is built and the level's textures are the loader's, so a set handed them
    // later has to catch up rather than leave every surface it already made untextured.
    const set: LevelMaterialSet = new LevelMaterialSet();
    const material: MeshStandardMaterial = set.claim(surfaceOf(7));

    expect(material.map).toBeNull();

    const textures: IMockLevelTextureSource = lookup("stone");

    set.setTextures(textures);

    expect(material.map).toBe(textures.get("stone")?.texture);
  });

  it("releases every material when the level goes", () => {
    const set: LevelMaterialSet = new LevelMaterialSet();

    set.claim(surfaceOf(1));
    set.claim(surfaceOf(2));
    set.dispose();

    expect(set.size).toBe(0);
  });

  // The reason the change is a set of references rather than a count: a sector arriving names a handful of textures,
  // and every material of the level that does not name one of them is already dressed correctly.
  it("re-dresses only the materials a change names", () => {
    const set: LevelMaterialSet = new LevelMaterialSet();
    const textures: IMockLevelTextureSource = lookup("stone", "rust");

    set.setTextures(mockLevelTextureSource());

    const stone: MeshStandardMaterial = set.claim(surfaceOf(1, { surface: mockSectorSurface({ shaderId: 1 }) }));
    const rust: MeshStandardMaterial = set.claim(
      surfaceOf(2, { surface: mockSectorSurface({ shaderId: 2, textureName: "rust" }) })
    );

    set.setTextures(textures);

    const untouched: number = rust.version;

    rust.map = null;
    textures.change(new Set(["stone"]));

    expect(stone.map).toBe(textures.get("stone")?.texture);
    // Left exactly as the caller put it: nothing re-dressed it, which is the whole point of the index.
    expect(rust.map).toBeNull();
    expect(rust.version).toBe(untouched);
  });

  it("re-dresses everything when the whole set goes", () => {
    const set: LevelMaterialSet = new LevelMaterialSet();
    const textures: IMockLevelTextureSource = lookup("stone");

    set.setTextures(textures);

    const material: MeshStandardMaterial = set.claim(surfaceOf(7));

    material.map = null;
    textures.change(null);

    expect(material.map).toBe(textures.get("stone")?.texture);
  });

  it("stops hearing about a set it no longer holds", () => {
    const set: LevelMaterialSet = new LevelMaterialSet();
    const first: IMockLevelTextureSource = lookup("stone");

    set.setTextures(first);

    const material: MeshStandardMaterial = set.claim(surfaceOf(7));

    set.setTextures(mockLevelTextureSource());
    first.change(null);

    expect(material.map).toBeNull();
  });

  // A material a departed sector was the last to name is disposed, and the index has to let go of it too or a later
  // change re-dresses a material three.js has already freed.
  it("forgets a material's references when it is released", () => {
    const set: LevelMaterialSet = new LevelMaterialSet();
    const textures: IMockLevelTextureSource = lookup("stone");

    set.setTextures(textures);

    const material: MeshStandardMaterial = set.claim(surfaceOf(7));

    set.retain([]);

    material.map = null;
    textures.change(new Set(["stone"]));

    expect(material.map).toBeNull();
  });
});
