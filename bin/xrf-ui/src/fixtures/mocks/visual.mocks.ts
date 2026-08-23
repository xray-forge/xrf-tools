import { createWorldSpec } from "@/core/assets/lib";
import { AssetTextureDescriptor, SelectedVisualDescription } from "@/core/bindings/types/xrf-app";
import {
  VisualBone,
  VisualBounds,
  VisualDescription,
  VisualGeometry,
  VisualSection,
  VisualSubmesh,
  VisualTextureDependency,
} from "@/core/bindings/types/xrf-visual";

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
 * Creates a bone fixture, placed in the bind pose when given a position.
 *
 * @param overrides - Field values to override.
 * @returns A bone fixture.
 */
export function mockVisualBone(overrides: Partial<VisualBone> = {}): VisualBone {
  return {
    name: "bip01",
    parent: "",
    parentIndex: null,
    bindPosition: null,
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
    world: createWorldSpec([]),
    textures: {},
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
