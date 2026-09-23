import {
  createRendererBox,
  createRendererSphere,
  ERendererCameraController,
  ERendererDebugView,
  ERendererDraw,
  ERendererTextureEncoding,
  IRendererGeometry,
  IRendererObject,
  IRendererOrbitCamera,
  IRendererSettings,
  IRendererSurface,
  TFrameRateLimit,
  TRendererTextureSource,
  withRendererTangentBasis,
} from "@xrf/renderer";

import {
  ETextureSurfaceAlpha,
  ETextureSurfaceShape,
  ITextureSurfaceFile,
  ITextureSurfaceFiles,
  ITextureSurfaceOptions,
} from "@/core/textures/lib/texture-surface";

/** How large each body is drawn, chosen so all three frame alike under one camera. */
const SHAPE_EXTENT: number = 2;

/** How thick the flat body is, as a fraction of its extent: enough to see it turn, too little to read as a box. */
const SLAB_THICKNESS: number = 0.02;

/** Everything the surface puts in the renderer, by the key it is held under. */
export const TEXTURE_SURFACE_KEYS = {
  base: "base",
  body: "body",
  bump: "bump",
  companion: "bump#",
  edge: "edge",
  face: "face",
} as const;

/** The camera the body is first seen from, and returned to. */
export const TEXTURE_SURFACE_CAMERA: IRendererOrbitCamera = {
  far: 100,
  fieldOfView: 45,
  kind: ERendererCameraController.ORBIT,
  near: 0.01,
  position: [0, 0, 5],
  target: [0, 0, 0],
};

/**
 * The slab's four edges and its back: dark and plain, so a turned slab reads as a slab rather than as the texture.
 */
export const TEXTURE_EDGE_SURFACE: IRendererSurface = {
  color: [0x1a / 255, 0x1a / 255, 0x1a / 255],
  draw: ERendererDraw.OPAQUE,
  textures: {},
};

/** How each reading of alpha is drawn: the engine's three answers for one. */
const ALPHA_DRAWS: Record<ETextureSurfaceAlpha, ERendererDraw> = {
  [ETextureSurfaceAlpha.IGNORED]: ERendererDraw.OPAQUE,
  [ETextureSurfaceAlpha.CUT_OUT]: ERendererDraw.CUT_OUT,
  [ETextureSurfaceAlpha.BLENDED]: ERendererDraw.BLENDED,
};

/**
 * @param shape - Body to build.
 * @returns It, with the tangent basis a bump pair rotates through.
 */
export function createTextureSurfaceGeometry(shape: ETextureSurfaceShape): IRendererGeometry {
  switch (shape) {
    case ETextureSurfaceShape.PLANE:
      return withRendererTangentBasis(createRendererBox(SHAPE_EXTENT, SHAPE_EXTENT, SHAPE_EXTENT * SLAB_THICKNESS));

    case ETextureSurfaceShape.SPHERE:
      return withRendererTangentBasis(createRendererSphere(SHAPE_EXTENT / 2, 96, 64));

    case ETextureSurfaceShape.CUBE:
      return withRendererTangentBasis(createRendererBox(SHAPE_EXTENT * 0.8, SHAPE_EXTENT * 0.8, SHAPE_EXTENT * 0.8));
  }
}

/**
 * The surface the texture is drawn with.
 *
 * @param files - What was read for the texture.
 * @param options - How it is being looked at.
 * @returns The surface, naming the base and the pair where they were read.
 */
export function toTextureSurface(files: ITextureSurfaceFiles, options: ITextureSurfaceOptions): IRendererSurface {
  return {
    draw: ALPHA_DRAWS[options.alpha],
    textures: {
      base: files.base ? TEXTURE_SURFACE_KEYS.base : undefined,
      bump: files.bump ? TEXTURE_SURFACE_KEYS.bump : undefined,
      bumpCompanion: files.bump ? TEXTURE_SURFACE_KEYS.companion : undefined,
    },
    tiling: options.tiling,
  };
}

/**
 * The body as an object: which surface each face draws, and the proportions it is stretched to.
 *
 * @param shape - The body.
 * @param aspect - Width over height of the texture on it.
 * @returns The object.
 */
export function toTextureSurfaceObject(shape: ETextureSurfaceShape, aspect: number): IRendererObject {
  const { edge, face } = TEXTURE_SURFACE_KEYS;

  if (shape === ETextureSurfaceShape.SPHERE) {
    return { geometry: TEXTURE_SURFACE_KEYS.body, surfaces: [face] };
  }

  // A box draws a group per face, and the cube carries the texture on every one of them.
  if (shape === ETextureSurfaceShape.CUBE) {
    return { geometry: TEXTURE_SURFACE_KEYS.body, surfaces: [face, face, face, face, face, face] };
  }

  const width: number = aspect >= 1 ? 1 : aspect;
  const height: number = aspect >= 1 ? 1 / aspect : 1;

  // The box's faces run +x, -x, +y, -y, +z, -z, and the texture belongs on the one facing the camera. Depth is left
  // alone, so a slab keeps one thickness whatever proportions its texture has.
  return {
    geometry: TEXTURE_SURFACE_KEYS.body,
    matrix: [width, 0, 0, 0, 0, height, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1],
    surfaces: [edge, edge, edge, edge, face, edge],
  };
}

/**
 * A file as the renderer takes it, copied: the renderer is handed the bytes, and the surface keeps its own.
 *
 * @param file - The file as it was read.
 * @returns Its bytes, and how they are encoded.
 */
export function toTextureSurfaceSource(file: ITextureSurfaceFile): TRendererTextureSource {
  const bytes: ArrayBuffer = file.bytes.slice(0);

  return file.isDecoded
    ? { bytes, encoding: ERendererTextureEncoding.IMAGE, type: "image/png" }
    : { bytes, encoding: ERendererTextureEncoding.DDS };
}

/**
 * @param options - How the texture is being looked at.
 * @param frameRateLimit - How often the application lets a view redraw.
 * @returns The renderer's settings for it.
 */
export function toTextureRendererSettings(
  options: ITextureSurfaceOptions,
  frameRateLimit: TFrameRateLimit
): IRendererSettings {
  return {
    // Transparent, so the checkerboard the frame already draws shows wherever the texture's alpha does.
    backdrop: null,
    debugView: ERendererDebugView.FINAL,
    frameRateLimit,
    isBumped: options.isBumped,
    isLit: options.isLit,
    tonemapScale: 1,
  };
}
