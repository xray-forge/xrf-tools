import {
  ERenderAntialiasing,
  ERenderDebugView,
  ERenderSurfaceColor,
  RenderAntialiasing,
  RenderDebugView,
  RenderSurfaceColor,
} from "@/core/ipc/types/xrf-renderer";

/**
 * What the level viewport shows of its surfaces: the finished frame, the frame with every surface clay or tinted by
 * its shader, or one of the targets the frame is built from.
 */
export enum ELevelShading {
  FINAL = "final",
  CLAY = "clay",
  SHADER = "shader",
  ALBEDO = "albedo",
  GLOSS = "gloss",
  NORMAL = "normal",
  BUMPS = "bumps",
  PARALLAX = "parallax",
  HEIGHTS = "heights",
  SAMPLED_MIP = "sampledMip",
  ALBEDO_RANGE = "albedoRange",
  OVERDRAW = "overdraw",
  CASCADES = "cascades",
  LAMPS_PER_CLUSTER = "lampsPerCluster",
  LUMINANCE = "luminance",
  HEMI = "hemi",
  SUN = "sun",
  MATERIAL = "material",
  DEPTH = "depth",
  LIGHT = "light",
  AMBIENT_OCCLUSION = "ambientOcclusion",
  INDIRECT_LIGHT = "indirectLight",
  REFLECTIONS = "reflections",
  REFLECTION_ENDS = "reflectionEnds",
  MOTION = "motion",
  FSR_MASKS = "fsrMasks",
  FSR_HISTORY_TRUST = "fsrHistoryTrust",
  FSR_LOCKS = "fsrLocks",
  FSR_LUMA_INSTABILITY = "fsrLumaInstability",
  FSR_FRAME_SHARE = "fsrFrameShare",
}

/** One shading as the toolbar offers it. */
export interface ILevelShadingChoice {
  value: ELevelShading;
  label: string;
  /** The heading it is listed under. */
  section: "Frame" | "Surfaces" | "Light" | "Temporal";
  /** One line on what it shows. */
  description: string;
  /** The surfaces' colour it draws with. */
  surfaceColor: RenderSurfaceColor;
  /** The picture it shows. */
  debugView: RenderDebugView;
}

/** FSR's own targets, offered only while FSR resolves the frame. */
const FSR_SHADINGS: ReadonlySet<ELevelShading> = new Set([
  ELevelShading.FSR_MASKS,
  ELevelShading.FSR_HISTORY_TRUST,
  ELevelShading.FSR_LOCKS,
  ELevelShading.FSR_LUMA_INSTABILITY,
  ELevelShading.FSR_FRAME_SHARE,
]);

/**
 * Every shading, in the order offered: the frame's three looks and its overdraw, the surfaces' targets, the light's,
 * then what the frame carries over from the last one.
 */
export const LEVEL_SHADINGS: ReadonlyArray<ILevelShadingChoice> = (
  [
    {
      section: "Frame",
      label: "Final frame",
      description: "The finished frame, every pass applied.",
      value: ELevelShading.FINAL,
      debugView: ERenderDebugView.FINAL,
    },
    {
      section: "Frame",
      label: "Clay",
      description: "One grey for every surface, so the shapes and the light read alone.",
      value: ELevelShading.CLAY,
      debugView: ERenderDebugView.FINAL,
    },
    {
      section: "Frame",
      label: "Shader colours",
      description: "Each surface tinted by its shader, so neighbouring surfaces tell apart.",
      value: ELevelShading.SHADER,
      debugView: ERenderDebugView.FINAL,
    },
    {
      section: "Frame",
      label: "Overdraw",
      description: "How many surfaces were drawn over each pixel, hidden or not: blue one, to red at eight.",
      value: ELevelShading.OVERDRAW,
      debugView: ERenderDebugView.OVERDRAW,
    },
    {
      section: "Surfaces",
      label: "Albedo",
      description: "Each surface's own colour, before any light.",
      value: ELevelShading.ALBEDO,
      debugView: ERenderDebugView.ALBEDO,
    },
    {
      section: "Surfaces",
      label: "Albedo range",
      description: "Each surface's colour as linear albedo: grey within 3% to 90%, blue darker, red brighter.",
      value: ELevelShading.ALBEDO_RANGE,
      debugView: ERenderDebugView.ALBEDO_RANGE,
    },
    {
      section: "Surfaces",
      label: "Gloss",
      description: "How glossy each surface is, from black for matte to white.",
      value: ELevelShading.GLOSS,
      debugView: ERenderDebugView.GLOSS,
    },
    {
      section: "Surfaces",
      label: "Normal",
      description: "The way each surface faces, relative to the camera, as colour.",
      value: ELevelShading.NORMAL,
      debugView: ERenderDebugView.NORMAL,
    },
    {
      section: "Surfaces",
      label: "Bumps",
      description: "Each bumped surface's bump normal as colour, lavender where flat; red where a surface has no bump.",
      value: ELevelShading.BUMPS,
      debugView: ERenderDebugView.BUMPS,
    },
    {
      section: "Surfaces",
      label: "Parallax",
      description: "How far parallax shifts each texture, across in red and down in green; grey where none is taken.",
      value: ELevelShading.PARALLAX,
      debugView: ERenderDebugView.PARALLAX,
    },
    {
      section: "Surfaces",
      label: "Heights",
      description: "The heights parallax reads, from black low to white high; dim blue where a surface has none.",
      value: ELevelShading.HEIGHTS,
      debugView: ERenderDebugView.HEIGHTS,
    },
    {
      section: "Surfaces",
      label: "Sampled mip",
      description:
        "The mip each surface's texture is read at: green its full size, blue magnified, yellow to red smaller.",
      value: ELevelShading.SAMPLED_MIP,
      debugView: ERenderDebugView.SAMPLED_MIP,
    },
    {
      section: "Surfaces",
      label: "Material",
      description: "The lighting model each surface is shaded with, one grey a model.",
      value: ELevelShading.MATERIAL,
      debugView: ERenderDebugView.MATERIAL,
    },
    {
      section: "Surfaces",
      label: "Depth",
      description: "The distance from the camera, from dark near to light far.",
      value: ELevelShading.DEPTH,
      debugView: ERenderDebugView.DEPTH,
    },
    {
      section: "Light",
      label: "Baked hemisphere",
      description: "How much of the sky each surface sees, as the level compiler baked it.",
      value: ELevelShading.HEMI,
      debugView: ERenderDebugView.HEMI,
    },
    {
      section: "Light",
      label: "Baked sun",
      description: "How much of the sun each surface gets, as the level compiler baked it.",
      value: ELevelShading.SUN,
      debugView: ERenderDebugView.SUN,
    },
    {
      section: "Light",
      label: "Accumulated light",
      description: "The light the sun and the lamps add up to, before the surfaces' colour.",
      value: ELevelShading.LIGHT,
      debugView: ERenderDebugView.LIGHT,
    },
    {
      section: "Light",
      label: "Shadow cascades",
      description:
        "The sun shadow map shading each pixel, darker where shadowed; striped where the grass casts shadows.",
      value: ELevelShading.CASCADES,
      debugView: ERenderDebugView.CASCADES,
    },
    {
      section: "Light",
      label: "Lamps per cluster",
      description: "How many lamps the cluster of each pixel lists: blue one, to red at sixteen.",
      value: ELevelShading.LAMPS_PER_CLUSTER,
      debugView: ERenderDebugView.LAMPS_PER_CLUSTER,
    },
    {
      section: "Light",
      label: "Ambient occlusion",
      description: "How much ambient light the screen-space occlusion leaves; white where it is off.",
      value: ELevelShading.AMBIENT_OCCLUSION,
      debugView: ERenderDebugView.AMBIENT_OCCLUSION,
    },
    {
      section: "Light",
      label: "Indirect light",
      description: "The bounced light as it would light a white surface; black where none is gathered.",
      value: ELevelShading.INDIRECT_LIGHT,
      debugView: ERenderDebugView.INDIRECT_LIGHT,
    },
    {
      section: "Light",
      label: "Reflections",
      description: "What each reflection ray met; deep blue where it met nothing, black where none is traced.",
      value: ELevelShading.REFLECTIONS,
      debugView: ERenderDebugView.REFLECTIONS,
    },
    {
      section: "Light",
      label: "Reflection rays",
      description: "How each reflection ray ended, one colour a reason; black where none is traced.",
      value: ELevelShading.REFLECTION_ENDS,
      debugView: ERenderDebugView.REFLECTION_ENDS,
    },
    {
      section: "Light",
      label: "Luminance",
      description: "The scene's brightness in stops from middle grey: grey at it, blue below, green to red above.",
      value: ELevelShading.LUMINANCE,
      debugView: ERenderDebugView.LUMINANCE,
    },
    {
      section: "Temporal",
      label: "Motion",
      description: "How far each surface moved on the screen since the last frame: red across, green down, grey still.",
      value: ELevelShading.MOTION,
      debugView: ERenderDebugView.MOTION,
    },
    {
      section: "Temporal",
      label: "FSR masks",
      description:
        "FSR's masks: how reactive each pixel is in red, how disoccluded in green, how far its shading changed in blue.",
      value: ELevelShading.FSR_MASKS,
      debugView: ERenderDebugView.FSR_MASKS,
    },
    {
      section: "Temporal",
      label: "FSR history trust",
      description: "How far FSR trusts each pixel's history, from black for none to white; red where it starts afresh.",
      value: ELevelShading.FSR_HISTORY_TRUST,
      debugView: ERenderDebugView.FSR_HISTORY_TRUST,
    },
    {
      section: "Temporal",
      label: "FSR locks",
      description: "FSR's locks on thin features in green; red where a lock was found this frame.",
      value: ELevelShading.FSR_LOCKS,
      debugView: ERenderDebugView.FSR_LOCKS,
    },
    {
      section: "Temporal",
      label: "FSR luma instability",
      description: "How strongly FSR finds each pixel's brightness flickering back to an earlier frame's.",
      value: ELevelShading.FSR_LUMA_INSTABILITY,
      debugView: ERenderDebugView.FSR_LUMA_INSTABILITY,
    },
    {
      section: "Temporal",
      label: "FSR frame share",
      description:
        "How much of each pixel FSR takes from this frame rather than its history, from black for none to white.",
      value: ELevelShading.FSR_FRAME_SHARE,
      debugView: ERenderDebugView.FSR_FRAME_SHARE,
    },
  ] satisfies ReadonlyArray<Omit<ILevelShadingChoice, "surfaceColor">>
).map((choice) => ({ ...choice, surfaceColor: toSurfaceColor(choice.value) }));

/** The same without FSR's own targets, for a frame FSR does not draw. */
const LEVEL_SHADINGS_WITHOUT_FSR: ReadonlyArray<ILevelShadingChoice> = LEVEL_SHADINGS.filter(
  (choice) => !FSR_SHADINGS.has(choice.value)
);

/**
 * @param shading - What the viewport shows.
 * @returns How it is offered: its label, section, description, the surfaces' colour and the picture.
 */
export function getLevelShading(shading: ELevelShading): ILevelShadingChoice {
  return LEVEL_SHADINGS.find((choice) => choice.value === shading) ?? LEVEL_SHADINGS[0];
}

/**
 * @param antialiasing - What smooths the view's frame.
 * @returns The shadings offered with it: FSR's own targets only while FSR resolves the frame.
 */
export function getLevelShadings(antialiasing: RenderAntialiasing): ReadonlyArray<ILevelShadingChoice> {
  return antialiasing === ERenderAntialiasing.FSR ? LEVEL_SHADINGS : LEVEL_SHADINGS_WITHOUT_FSR;
}

/**
 * @param shading - What the view asks to show.
 * @param antialiasing - What smooths the view's frame.
 * @returns What it shows: the final frame in place of one of FSR's targets while FSR does not resolve the frame.
 */
export function toShownLevelShading(shading: ELevelShading, antialiasing: RenderAntialiasing): ELevelShading {
  return FSR_SHADINGS.has(shading) && antialiasing !== ERenderAntialiasing.FSR ? ELevelShading.FINAL : shading;
}

function toSurfaceColor(shading: ELevelShading): RenderSurfaceColor {
  switch (shading) {
    case ELevelShading.CLAY:
      return ERenderSurfaceColor.CLAY;

    case ELevelShading.SHADER:
      return ERenderSurfaceColor.SHADER;

    default:
      return ERenderSurfaceColor.TEXTURED;
  }
}
