import { createRoots } from "@/core/assets/lib";
import { AssetTextureDescriptor, SelectedVisualDescription } from "@/core/bindings/types/xrf-app";
import { Vector3d } from "@/core/bindings/types/xrf-db";
import { XrayMaterialDescriptor } from "@/core/bindings/types/xrf-material";
import {
  VisualBone,
  VisualBounds,
  VisualDescription,
  VisualGeometry,
  VisualMotionBake,
  VisualSection,
  VisualSubmesh,
  VisualTextureDependency,
  VisualTransform,
} from "@/core/bindings/types/xrf-visual";
import { MOTION_DEFAULT_SPEED, MOTION_SAMPLE_FPS } from "@/core/visuals/lib/visual-motion";
import { FLOATS_PER_BONE, IVisualModelViews } from "@/core/visuals/lib/visual-views";

const ALIGNMENT: number = 4;

/**
 * Packs sections the way the rust builder does, so a test reasons about a real buffer.
 *
 * Every section is padded to a four byte boundary because that is the invariant typed array views rely
 * on; a fixture that skipped it would let a broken view constructor pass.
 */
export class MockVisualBuffer {
  private readonly chunks: Array<Uint8Array> = [];

  private length: number = 0;

  public pushFloats(values: Array<number>): VisualSection {
    const bytes: Uint8Array = new Uint8Array(new Float32Array(values).buffer);

    return this.push(bytes);
  }

  public pushIndices(values: Array<number>): VisualSection {
    const bytes: Uint8Array = new Uint8Array(new Uint16Array(values).buffer);

    return this.push(bytes);
  }

  public toArrayBuffer(): ArrayBuffer {
    // Allocated as an `ArrayBuffer` and written through a view, rather than taking `.buffer` off a typed
    // array, because that property is `ArrayBufferLike` and could be shared memory.
    const buffer: ArrayBuffer = new ArrayBuffer(this.length);
    const bytes: Uint8Array = new Uint8Array(buffer);

    let offset: number = 0;

    for (const chunk of this.chunks) {
      bytes.set(chunk, offset);
      offset += chunk.byteLength;
    }

    return buffer;
  }

  public get byteLength(): number {
    return this.length;
  }

  private push(bytes: Uint8Array): VisualSection {
    const padding: number = this.length % ALIGNMENT === 0 ? 0 : ALIGNMENT - (this.length % ALIGNMENT);

    if (padding > 0) {
      this.chunks.push(new Uint8Array(padding));
      this.length += padding;
    }

    const section: VisualSection = { byteOffset: this.length, byteLength: bytes.byteLength };

    this.chunks.push(bytes);
    this.length += bytes.byteLength;

    return section;
  }
}

/**
 * Creates a visual bounds fixture.
 *
 * @param overrides - Field values to override.
 * @returns A visual bounds fixture.
 */
export function mockVisualBounds(overrides: Partial<VisualBounds> = {}): VisualBounds {
  return {
    boundingBox: { min: { x: -1, y: -1, z: -1 }, max: { x: 1, y: 1, z: 1 } },
    boundingSphere: { center: { x: 0, y: 0, z: 0 }, radius: 1.5 },
    ...overrides,
  };
}

/**
 * Creates a packed submesh fixture and appends its attributes to the provided buffer.
 *
 * @param buffer - Buffer the attributes are packed into.
 * @param overrides - Submesh field values to override.
 * @param geometryOverrides - Geometry field values to override, such as the draw range.
 * @returns A submesh whose sections address the packed bytes.
 */
export function mockPackedSubmesh(
  buffer: MockVisualBuffer,
  overrides: Partial<Omit<VisualSubmesh, "content">> = {},
  geometryOverrides: Partial<VisualGeometry> = {}
): VisualSubmesh {
  const positions: VisualSection = buffer.pushFloats([0, 0, 0, 1, 0, 0, 0, 1, 0]);
  const normals: VisualSection = buffer.pushFloats([0, 0, 1, 0, 0, 1, 0, 0, 1]);
  const uvs: VisualSection = buffer.pushFloats([0, 0, 1, 0, 0, 1]);
  const indices: VisualSection = buffer.pushIndices([0, 1, 2]);

  return {
    index: 0,
    modelType: 5,
    modelTypeLabel: "MT_SKELETON_GEOMDEF_ST",
    textureName: "wpn\\wpn_ak74",
    shaderName: "models\\weapons",
    ...overrides,
    content: {
      kind: "packed",
      geometry: {
        vertexCount: 3,
        indexCount: 3,
        positions,
        normals,
        uvs,
        indices,
        skin: null,
        detailLevels: [{ start: 0, count: 3 }],
        bounds: mockVisualBounds(),
        ...geometryOverrides,
      },
    },
  };
}

/**
 * Creates a skipped submesh fixture, which packs no bytes.
 *
 * @param overrides - Field values to override.
 * @returns A submesh carrying the reason it produced no geometry.
 */
export function mockSkippedSubmesh(overrides: Partial<Omit<VisualSubmesh, "content">> = {}): VisualSubmesh {
  return {
    index: 1,
    modelType: 5,
    modelTypeLabel: "MT_SKELETON_GEOMDEF_ST",
    textureName: null,
    shaderName: null,
    ...overrides,
    content: { kind: "skipped", cause: "unsupported", reason: "Carries no geometry chunk" },
  };
}

/**
 * Creates a visual description fixture.
 *
 * @param overrides - Field values to override.
 * @returns A visual description fixture.
 */
export function mockVisualDescription(overrides: Partial<VisualDescription> = {}): VisualDescription {
  return {
    version: 4,
    modelType: 3,
    modelTypeLabel: "MT_SKELETON_ANIM",
    shaderId: 0,
    sourceFile: "x:\\rawdata\\objects\\wpn_ak74.object",
    declaredBounds: mockVisualBounds(),
    computedBounds: mockVisualBounds(),
    submeshes: [],
    bones: [],
    motionRefs: [],
    embeddedMotions: [],
    bufferLength: 0,
    ...overrides,
  };
}

/**
 * Creates an unrotated transform at one position, which is what most tests mean by "a bone is here".
 *
 * @param translation - Where the transform places its origin.
 * @returns A transform fixture with an identity basis.
 */
export function mockVisualTransform(translation: Vector3d): VisualTransform {
  return {
    i: { x: 1, y: 0, z: 0 },
    j: { x: 0, y: 1, z: 0 },
    k: { x: 0, y: 0, z: 1 },
    c: translation,
  };
}

/**
 * Creates a bone fixture, placed in the bind pose when given a transform.
 *
 * @param overrides - Field values to override.
 * @returns A bone fixture.
 */
export function mockVisualBone(overrides: Partial<VisualBone> = {}): VisualBone {
  return {
    name: "bip01",
    parent: "",
    parentIndex: null,
    bindTransform: null,
    ...overrides,
  };
}

/**
 * Creates a selected visual fixture, pairing a description with the source it came from.
 *
 * @param overrides - Field values to override.
 * @returns A selected visual fixture.
 */
export function mockSelectedVisual(overrides: Partial<SelectedVisualDescription> = {}): SelectedVisualDescription {
  return {
    source: { kind: "file", path: "C:\\gamedata\\meshes\\wpn_ak74.ogf" },
    description: mockVisualDescription(),
    dependencies: { motions: [], textures: [] },
    roots: createRoots([]),
    textures: {},
    materials: {},
    texturesLtx: null,
    ...overrides,
  };
}

/**
 * Creates a material descriptor fixture: a bumped declaration whose pair resolved.
 *
 * @param overrides - Field values to override.
 * @returns A material descriptor fixture.
 */
export function mockMaterialDescriptor(overrides: Partial<XrayMaterialDescriptor> = {}): XrayMaterialDescriptor {
  return {
    descriptor: {
      container: { kind: "directory", relativePath: "textures\\wpn\\wpn_ak74.thm", root: "C:\\gamedata" },
      logicalPath: "textures\\wpn\\wpn_ak74.thm",
    },
    declaration: { kind: "declared", mode: "use", name: "wpn\\wpn_ak74_bump" },
    bump: {
      mode: "use",
      virtualHeight: 0.05,
      bump: {
        reference: "wpn\\wpn_ak74_bump",
        resolution: {
          kind: "resolved",
          step: "asset root",
          assets: [
            {
              container: { kind: "directory", relativePath: "textures\\wpn\\wpn_ak74_bump.dds", root: "C:\\gamedata" },
              logicalPath: "textures\\wpn\\wpn_ak74_bump.dds",
            },
          ],
        },
      },
      companion: {
        reference: "wpn\\wpn_ak74_bump#",
        resolution: {
          kind: "resolved",
          step: "asset root",
          assets: [
            {
              container: {
                kind: "directory",
                relativePath: "textures\\wpn\\wpn_ak74_bump#.dds",
                root: "C:\\gamedata",
              },
              logicalPath: "textures\\wpn\\wpn_ak74_bump#.dds",
            },
          ],
        },
      },
    },
    outcome: "bumped",
    detail: null,
    ...overrides,
  };
}

/**
 * Creates a located texture's descriptor fixture.
 *
 * @param overrides - Field values to override.
 * @returns A texture descriptor fixture.
 */
export function mockTextureDescriptor(overrides: Partial<AssetTextureDescriptor> = {}): AssetTextureDescriptor {
  return {
    size: 349_672,
    shape: { width: 512, height: 512, mipmapLevels: 10, format: "DXT5" },
    ...overrides,
  };
}

/**
 * Creates a resolved submesh texture dependency fixture.
 *
 * @param overrides - Field values to override.
 * @returns A texture dependency fixture that resolved to a file.
 */
export function mockTextureDependency(overrides: Partial<VisualTextureDependency> = {}): VisualTextureDependency {
  return {
    reference: "wpn\\wpn_ak74",
    resolution: {
      kind: "resolved",
      step: "asset root",
      assets: [
        {
          container: {
            kind: "directory",
            relativePath: "textures\\wpn\\wpn_ak74.dds",
            root: "C:\\gamedata",
          },
          logicalPath: "textures\\wpn\\wpn_ak74.dds",
        },
      ],
    },
    submeshIndex: 0,
    ...overrides,
  };
}

/**
 * Creates a baked motion fixture.
 *
 * The duration follows the frame count and the speed unless it is overridden, so a test asking for a longer or a
 * faster motion does not have to restate how long that makes it.
 *
 * @param overrides - Field values to override.
 * @returns A bake fixture.
 */
export function mockVisualMotionBake(overrides: Partial<VisualMotionBake> = {}): VisualMotionBake {
  const bake: VisualMotionBake = {
    name: "norm_walk_fwd_1",
    frameCount: 3,
    boneCount: 2,
    animatedBoneCount: 2,
    floatsPerBone: FLOATS_PER_BONE,
    duration: 0,
    speed: MOTION_DEFAULT_SPEED,
    ...overrides,
  };

  const speed: number = bake.speed && bake.speed > 0 ? bake.speed : MOTION_DEFAULT_SPEED;

  return { ...bake, duration: overrides.duration ?? bake.frameCount / (MOTION_SAMPLE_FPS * speed) };
}

/**
 * Creates the bone transforms one bake describes.
 *
 * Built from the bake rather than from counts of its own, because every consumer refuses bytes whose length disagrees
 * with the frame, bone and float counts it was told: a test producing the two separately could pin that guard against
 * a buffer it made up.
 *
 * @param bake - Bake the buffer has to match.
 * @param fill - Value written into every float of one frame, by default the frame's own index so a pose is
 *   identifiable.
 * @returns Frame major transforms of the length the bake reports.
 */
export function mockVisualMotionTransforms(
  bake: VisualMotionBake,
  fill: (frame: number) => number = (frame: number) => frame
): ArrayBuffer {
  const stride: number = bake.boneCount * bake.floatsPerBone;
  const transforms: Float32Array = new Float32Array(bake.frameCount * stride);

  for (let frame: number = 0; frame < bake.frameCount; frame += 1) {
    transforms.fill(fill(frame), frame * stride, (frame + 1) * stride);
  }

  return transforms.buffer as ArrayBuffer;
}

/**
 * One bone's transform, flattened the way both the bind buffer and a baked motion store it.
 *
 * An identity basis and a translation, because a test reading a posed bone is nearly always asking where it ended up
 * rather than which way it faces. `visual-views.test.ts` writes this layout out literally on purpose - it is the test
 * that proves it - so this builder is for the tests that consume the layout rather than assert it.
 *
 * @param translation - Where this bone sits, in all three axes.
 * @returns Twelve floats: `i`, `j`, `k`, then `c`.
 */
export function mockVisualBoneFloats(translation: number): Array<number> {
  return [1, 0, 0, 0, 1, 0, 0, 0, 1, translation, translation, translation];
}

/**
 * Creates model views, the shape every scene-layer component is handed.
 *
 * Empty by default - no geometry, no skeleton - so a test naming one field says exactly what it is about.
 *
 * @param overrides - Views to replace.
 * @returns Model views a scene can be built from.
 */
export function mockVisualModelViews(overrides: Partial<IVisualModelViews> = {}): IVisualModelViews {
  return {
    submeshes: [],
    fit: { center: [0, 0, 0], radius: 1 },
    skeleton: null,
    skeletonPairs: null,
    skeletonBinds: null,
    vertexCount: 0,
    levelCount: 1,
    ...overrides,
  };
}
