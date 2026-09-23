import {
  DEFAULT_RENDERER_LIGHTING,
  ERendererCameraController,
  ERendererDebugView,
  ERendererTextureEncoding,
  IRendererBounds,
  IRendererFlyCamera,
  IRendererFog,
  IRendererGeometry,
  IRendererLighting,
  IRendererObject,
  IRendererSettings,
  IRendererSurface,
  TFrameRateLimit,
  toneMapReinhard,
  TRendererColor,
  TRendererTextureSource,
} from "@xrf/renderer";
import { Nullable } from "@xrf/types";

import { SectorSurface, VisualBounds } from "@/core/ipc/types/xrf-visual";
import { ILevelCameraOptions } from "@/core/level/lib/camera/level-camera-options";
import { ILevelViewpoint, toLevelStartViewpoint } from "@/core/level/lib/camera/level-viewpoint";
import { ILevelLighting } from "@/core/level/lib/lighting/level-lighting";
import { ILevelRenderConfig } from "@/core/level/lib/render/level-render-config";
import { ILevelTextureDelivery } from "@/core/level/lib/render/level-render-protocol";
import { ISectorGeometryViews, ISectorInstanceViews, ISectorViews } from "@/core/level/lib/sector/level-sector-views";
import { ILevelSurfaceOptions } from "@/core/level/lib/surface/level-surface-options";
import { ILevelSurfaceRender } from "@/core/level/lib/surface/level-surface-render";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { toRendererLighting } from "@/core/render/lib/lighting/render-lighting";

/** The keys a level is put to the renderer under. */
export const LEVEL_RENDER_KEYS = {
  axes: "axes",
  extent: "extent",
  extentBox: "extent-box",
  grid: "grid",
  /** One mesh a sector stands in many places. */
  instance: (sector: number, index: number): string => `sector:${sector}:instance:${index}`,
  /** Everything a sector bakes in place, one geometry with a group per surface. */
  sector: (sector: number): string => `sector:${sector}`,
  sun: "sun",
  /** One shader table entry, which every sector drawing it shares. */
  surface: (shaderId: number): string => `surface:${shaderId}`,
} as const;

/**
 * `default_clear`'s noon fog (`configs/environment/weathers/default_clear.ltx`, `[12:00:00]`): starts at a tenth of
 * 85% of its distance and is total at the far plane, 350 metres.
 */
export const LEVEL_NOON_FOG: IRendererFog = {
  color: [0.304609, 0.328138, 0.367354],
  density: 0.9,
  distance: 350,
};

/** Turns of the golden angle, which spreads consecutive shader ids rather than grouping them into near hues. */
const HUE_STEP: number = 137.508;

/** The stand-in's side, in texels: small, since it tiles, and all that matters is that it reads as a pattern. */
const CHECKER_SIZE: number = 16;

/** Magenta and near black, the colours a missing texture has meant since before any of this. */
const CHECKER_COLORS: ReadonlyArray<readonly [number, number, number]> = [
  [255, 0, 255],
  [16, 16, 16],
];

/**
 * @param views - A sector as it arrived.
 * @returns What it bakes in place, as the renderer takes it: views over the one buffer it arrived in.
 */
export function toLevelSectorGeometry(views: ISectorViews): IRendererGeometry {
  return {
    ...toLevelGeometry(views.geometry),
    bounds: toLevelBounds(views.bounds),
    groups: views.sections.map((section, slot: number) => ({ count: section.count, slot, start: section.start })),
  };
}

/**
 * @param instance - One mesh a sector stands in many places.
 * @returns The mesh, in its own space.
 */
export function toLevelInstanceGeometry(instance: ISectorInstanceViews): IRendererGeometry {
  return { ...toLevelGeometry(instance.geometry), groups: [] };
}

/**
 * @param views - A sector as it arrived.
 * @returns What draws its baked geometry: one surface per section, in the order its groups name them.
 */
export function toLevelSectorObject(views: ISectorViews): IRendererObject {
  return {
    geometry: LEVEL_RENDER_KEYS.sector(views.sector),
    surfaces: views.sections.map((section) => LEVEL_RENDER_KEYS.surface(section.surface.shaderId)),
  };
}

/**
 * @param sector - The sector the mesh belongs to.
 * @param index - Its position among the sector's instanced meshes.
 * @param instance - The mesh and the places it stands.
 * @returns What draws it in every one of them.
 */
export function toLevelInstanceObject(sector: number, index: number, instance: ISectorInstanceViews): IRendererObject {
  return {
    geometry: LEVEL_RENDER_KEYS.instance(sector, index),
    // The engine stores a row-vector matrix row major and the renderer takes a column-vector one column major: the
    // same sixteen floats, so nothing is rearranged.
    instances: { hemi: instance.hemi, transforms: instance.transforms },
    surfaces: [LEVEL_RENDER_KEYS.surface(instance.surface.shaderId)],
  };
}

/**
 * One shader table entry, as the renderer draws it.
 *
 * @param surface - What the level's table names for it.
 * @param render - What its blender compiles to.
 * @param options - What the toolbar has switched on.
 * @returns The surface.
 */
export function toLevelSurface(
  surface: SectorSurface,
  render: ILevelSurfaceRender,
  options: ILevelSurfaceOptions
): IRendererSurface {
  const base: Nullable<string> = options.isTextured ? surface.textureName : null;
  const detail = options.isTextured ? render.detail : null;

  return {
    alphaReference: render.alphaReference,
    // An untextured surface takes its entry's colour, so a level with textures off is still read surface by surface.
    color: base ? undefined : toLevelSurfaceColor(surface.shaderId),
    detailScale: detail?.scale,
    draw: render.draw,
    isLit: render.isLit,
    isWallmark: render.isWallmark || undefined,
    textures: {
      base: base ?? undefined,
      detail: detail?.reference,
      // The row's third texture, which `uber_deffer` binds as `s_hemi`: the second is R1's baked colour.
      hemi: surface.hemi ?? undefined,
    },
  };
}

/**
 * A stable colour per shader table entry, so one surface is the same colour in every sector of the level.
 *
 * @param shaderId - Entry of the level's shader table.
 * @returns Raw red, green and blue, from a hue derived from the id rather than assigned in arrival order.
 */
export function toLevelSurfaceColor(shaderId: number): TRendererColor {
  const hue: number = (shaderId * HUE_STEP) % 360;

  return toRgb(hue, 0.45, 0.6);
}

/**
 * @param delivery - One file the loader read, or the reason it could not.
 * @returns What the renderer uploads: the file, the backend's picture of it, or a checker standing in.
 */
export function toLevelTextureSource(delivery: ILevelTextureDelivery): TRendererTextureSource {
  if (delivery.reason) {
    return createLevelCheckerSource();
  }

  return delivery.isDecoded
    ? { bytes: delivery.bytes, encoding: ERendererTextureEncoding.IMAGE, type: "image/png" }
    : { bytes: delivery.bytes, encoding: ERendererTextureEncoding.DDS };
}

/**
 * A stand-in that survives whatever would hide it: opaque, since the surfaces most needing one are cut out, and
 * sampled nearest, since a filtered checker at distance is the flat grey it exists to differ from.
 *
 * @returns The checker, magenta and near black.
 */
export function createLevelCheckerSource(): TRendererTextureSource {
  const bytes: Uint8Array<ArrayBuffer> = new Uint8Array(CHECKER_SIZE * CHECKER_SIZE * 4);

  for (let y = 0; y < CHECKER_SIZE; y += 1) {
    for (let x = 0; x < CHECKER_SIZE; x += 1) {
      bytes.set([...CHECKER_COLORS[((x >> 1) + (y >> 1)) % 2], 255], (y * CHECKER_SIZE + x) * 4);
    }
  }

  return {
    bytes: bytes.buffer,
    encoding: ERendererTextureEncoding.RGBA,
    height: CHECKER_SIZE,
    isNearest: true,
    width: CHECKER_SIZE,
  };
}

/**
 * Where a level opens, flown by the toolbar's speeds.
 *
 * @param bounds - The level's extent, or null for a level that reports none.
 * @param options - The camera the toolbar asks for.
 * @param config - The near and far planes.
 * @returns The camera.
 */
export function toLevelCamera(
  bounds: Nullable<VisualBounds>,
  options: ILevelCameraOptions,
  config: ILevelRenderConfig
): IRendererFlyCamera {
  const { position, target }: ILevelViewpoint = toLevelStartViewpoint(bounds);

  return {
    boost: options.boost,
    far: config.cameraFar,
    fieldOfView: options.fieldOfView,
    kind: ERendererCameraController.FLY,
    near: config.cameraNear,
    position: [position.x, position.y, position.z],
    sensitivity: options.sensitivity,
    speed: options.speed,
    target: [target.x, target.y, target.z],
  };
}

/**
 * @param lighting - The level's light, as its controls set it.
 * @param isFogged - Whether the noon fog is drawn.
 * @returns The engine's noon, pointed and scaled by those controls.
 */
export function toLevelRendererLighting(lighting: ILevelLighting, isFogged: boolean): IRendererLighting {
  return { ...toRendererLighting(lighting, DEFAULT_RENDERER_LIGHTING), fog: isFogged ? LEVEL_NOON_FOG : null };
}

/**
 * @param options - The toolbar's toggles.
 * @param lighting - The level's light, whose hemisphere strength the baked light toggle gates.
 * @param frameRateLimit - How often the application lets a view redraw.
 * @param config - The backdrop.
 * @returns The renderer's settings.
 */
export function toLevelRendererSettings(
  options: ILevelViewOptions,
  lighting: ILevelLighting,
  frameRateLimit: TFrameRateLimit,
  config: ILevelRenderConfig
): IRendererSettings {
  return {
    // Fogged, the sky is what total fog comes to, so the horizon does not end in a line.
    backdrop: options.isFogged ? toToneMappedHex(LEVEL_NOON_FOG.color, 1) : config.backgroundColor,
    debugView: ERendererDebugView.FINAL,
    frameRateLimit,
    hemiStrength: options.isLit ? lighting.hemiStrength : 0,
    isBumped: true,
    isLit: true,
    isWireframe: options.isWireframe,
    tonemapScale: 1,
  };
}

function toLevelGeometry(geometry: ISectorGeometryViews): Omit<IRendererGeometry, "groups"> {
  return {
    hemi: geometry.hemi ?? undefined,
    index: geometry.indices,
    normal: geometry.normals ?? undefined,
    position: geometry.positions,
    uv: geometry.uvs ?? undefined,
    uv1: geometry.lightmapUvs ?? undefined,
  };
}

/** The packer's sphere, where it measured one: a non-finite radius crosses the wire as null. */
function toLevelBounds(bounds: Nullable<VisualBounds>): IRendererBounds | undefined {
  const sphere = bounds?.boundingSphere;

  if (!sphere || sphere.radius === null || sphere.radius === undefined) {
    return undefined;
  }

  return { center: [sphere.center.x ?? 0, sphere.center.y ?? 0, sphere.center.z ?? 0], radius: sphere.radius };
}

/** A raw colour through the engine's tonemap, as the hex a page writes. */
function toToneMappedHex(color: TRendererColor, scale: number): number {
  const [red, green, blue] = color.map((channel: number) =>
    Math.round(Math.min(1, toneMapReinhard(channel, scale)) * 255)
  );

  return (red << 16) | (green << 8) | blue;
}

/** Hue in degrees, saturation and lightness in `[0, 1]`, to raw red, green and blue. */
function toRgb(hue: number, saturation: number, lightness: number): TRendererColor {
  const reach: number = saturation * Math.min(lightness, 1 - lightness);

  function channel(offset: number): number {
    const turn: number = (offset + hue / 30) % 12;

    return lightness - reach * Math.max(-1, Math.min(turn - 3, 9 - turn, 1));
  }

  return [channel(0), channel(8), channel(4)];
}
