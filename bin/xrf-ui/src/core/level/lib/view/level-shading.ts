import {
  ERenderDebugView,
  ERenderSurfaceColor,
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
  HEMI = "hemi",
  SUN = "sun",
  MATERIAL = "material",
  DEPTH = "depth",
  LIGHT = "light",
  AMBIENT_OCCLUSION = "ambientOcclusion",
  INDIRECT_LIGHT = "indirectLight",
  MOTION = "motion",
}

/** One shading as the toolbar offers it. */
export interface ILevelShadingChoice {
  value: ELevelShading;
  label: string;
  /** The surfaces' colour it draws with. */
  surfaceColor: RenderSurfaceColor;
  /** The picture it shows. */
  debugView: RenderDebugView;
}

/** Every shading, in the order offered: the frame's three looks, then the targets in the order they are built. */
export const LEVEL_SHADINGS: ReadonlyArray<ILevelShadingChoice> = [
  { label: "Final frame", value: ELevelShading.FINAL, debugView: ERenderDebugView.FINAL },
  { label: "Clay", value: ELevelShading.CLAY, debugView: ERenderDebugView.FINAL },
  { label: "Shader colours", value: ELevelShading.SHADER, debugView: ERenderDebugView.FINAL },
  { label: "Albedo", value: ELevelShading.ALBEDO, debugView: ERenderDebugView.ALBEDO },
  { label: "Gloss", value: ELevelShading.GLOSS, debugView: ERenderDebugView.GLOSS },
  { label: "Normal", value: ELevelShading.NORMAL, debugView: ERenderDebugView.NORMAL },
  { label: "Baked hemisphere", value: ELevelShading.HEMI, debugView: ERenderDebugView.HEMI },
  { label: "Baked sun", value: ELevelShading.SUN, debugView: ERenderDebugView.SUN },
  { label: "Material", value: ELevelShading.MATERIAL, debugView: ERenderDebugView.MATERIAL },
  { label: "Depth", value: ELevelShading.DEPTH, debugView: ERenderDebugView.DEPTH },
  { label: "Accumulated light", value: ELevelShading.LIGHT, debugView: ERenderDebugView.LIGHT },
  { label: "Ambient occlusion", value: ELevelShading.AMBIENT_OCCLUSION, debugView: ERenderDebugView.AMBIENT_OCCLUSION },
  { label: "Indirect light", value: ELevelShading.INDIRECT_LIGHT, debugView: ERenderDebugView.INDIRECT_LIGHT },
  { label: "Motion", value: ELevelShading.MOTION, debugView: ERenderDebugView.MOTION },
].map((choice) => ({ ...choice, surfaceColor: toSurfaceColor(choice.value) }));

/**
 * @param shading - What the viewport shows.
 * @returns How it is offered: its label, the surfaces' colour and the picture.
 */
export function getLevelShading(shading: ELevelShading): ILevelShadingChoice {
  return LEVEL_SHADINGS.find((choice) => choice.value === shading) ?? LEVEL_SHADINGS[0];
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
