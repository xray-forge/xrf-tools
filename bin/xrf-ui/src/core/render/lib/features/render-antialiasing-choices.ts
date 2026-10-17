import {
  ERenderAntialiasing,
  ERenderDlssAvailability,
  ERenderTextureFiltering,
  RenderAntialiasing,
  RenderDlssAvailability,
  RenderTextureFiltering,
} from "@/core/ipc/types/xrf-renderer";
import { IChoiceFormRowOption } from "@/core/ui/form/ChoiceFormRow/choice-form-row-option";

const RENDER_ANTIALIASING_NAMES: Readonly<Record<RenderAntialiasing, string>> = {
  [ERenderAntialiasing.NONE]: "None",
  [ERenderAntialiasing.FXAA]: "FXAA",
  [ERenderAntialiasing.SMAA]: "SMAA",
  [ERenderAntialiasing.TAA]: "TAA",
  [ERenderAntialiasing.FSR]: "FSR 3.1",
  [ERenderAntialiasing.DLSS]: "DLSS",
};

const RENDER_ANTIALIASING_DESCRIPTIONS: Readonly<Record<RenderAntialiasing, string>> = {
  [ERenderAntialiasing.NONE]: "Draws every edge as it falls, unsmoothed.",
  [ERenderAntialiasing.FXAA]: "Smooths edges in one cheap pass, softening the image a little.",
  [ERenderAntialiasing.SMAA]: "Smooths edges by their shapes, sharp and stable.",
  [ERenderAntialiasing.TAA]: "Smooths edges over frames, foliage and thin geometry too.",
  [ERenderAntialiasing.FSR]: "Smooths and upscales over frames with AMD's FSR 3.1, steadiest on fine detail.",
  [ERenderAntialiasing.DLSS]:
    "Smooths and upscales with NVIDIA's learned model on an RTX GPU; FSR stands in elsewhere.",
};

/**
 * @param mode - An antialiasing mode.
 * @returns What it does, in a sentence.
 */
export function explainRenderAntialiasing(mode: RenderAntialiasing): string {
  return RENDER_ANTIALIASING_DESCRIPTIONS[mode];
}

/**
 * @param mode - An antialiasing mode.
 * @returns Its name as the settings say it.
 */
export function describeRenderAntialiasing(mode: RenderAntialiasing): string {
  return RENDER_ANTIALIASING_NAMES[mode];
}

/**
 * @param mode - The antialiasing asked for.
 * @param dlss - Whether the GPU drawing the frame upscales with DLSS.
 * @returns What smooths the frame there: FSR in place of DLSS on a GPU without it, as the renderer resolves it.
 */
export function toDrawnAntialiasing(mode: RenderAntialiasing, dlss: RenderDlssAvailability): RenderAntialiasing {
  return mode === ERenderAntialiasing.DLSS && dlss !== ERenderDlssAvailability.AVAILABLE
    ? ERenderAntialiasing.FSR
    : mode;
}

/**
 * @param mode - An antialiasing mode.
 * @returns Whether it resolves frames temporally (TAA, FSR, DLSS), which RCAS then sharpens at any size.
 */
export function isTemporalAntialiasing(mode: RenderAntialiasing): boolean {
  return mode === ERenderAntialiasing.TAA || mode === ERenderAntialiasing.FSR || mode === ERenderAntialiasing.DLSS;
}

/** Every antialiasing mode, in the order they are offered. */
export const RENDER_ANTIALIASING_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderAntialiasing>> = Object.values(
  ERenderAntialiasing
).map((value: RenderAntialiasing) => ({ label: describeRenderAntialiasing(value), value }));

const RENDER_TEXTURE_FILTERING_NAMES: Readonly<Record<RenderTextureFiltering, string>> = {
  [ERenderTextureFiltering.TRILINEAR]: "Trilinear",
  [ERenderTextureFiltering.ANISOTROPIC2]: "Anisotropic 2×",
  [ERenderTextureFiltering.ANISOTROPIC4]: "Anisotropic 4×",
  [ERenderTextureFiltering.ANISOTROPIC8]: "Anisotropic 8×",
  [ERenderTextureFiltering.ANISOTROPIC16]: "Anisotropic 16×",
};

const RENDER_TEXTURE_FILTERING_SHORT_NAMES: Readonly<Record<RenderTextureFiltering, string>> = {
  [ERenderTextureFiltering.TRILINEAR]: "Tri",
  [ERenderTextureFiltering.ANISOTROPIC2]: "2×",
  [ERenderTextureFiltering.ANISOTROPIC4]: "4×",
  [ERenderTextureFiltering.ANISOTROPIC8]: "8×",
  [ERenderTextureFiltering.ANISOTROPIC16]: "16×",
};

/** The same, named short enough for a toolbar popover's row: trilinear, then the anisotropy's samples. */
export const RENDER_TEXTURE_FILTERING_SHORT_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderTextureFiltering>> =
  Object.values(ERenderTextureFiltering).map((value: RenderTextureFiltering) => ({
    label: RENDER_TEXTURE_FILTERING_SHORT_NAMES[value],
    value,
  }));

/** How a level's surfaces may sample their textures, as the settings offer them. */
export const RENDER_TEXTURE_FILTERING_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderTextureFiltering>> =
  Object.values(ERenderTextureFiltering).map((value: RenderTextureFiltering) => ({
    label: RENDER_TEXTURE_FILTERING_NAMES[value],
    value,
  }));

/**
 * The game's texture qualities, `texture_lod`, by the Anomaly preset that sets each; the game's own default is Medium,
 * while every preset here sets Full.
 */
export const RENDER_TEXTURE_LOD_OPTIONS: ReadonlyArray<IChoiceFormRowOption<string>> = [
  { label: "Full (extreme)", value: "0" },
  { label: "High", value: "1" },
  { label: "Medium (game default)", value: "2" },
  { label: "Low", value: "3" },
  { label: "Lowest", value: "4" },
];
