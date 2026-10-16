import {
  ERenderAntialiasing,
  ERenderTextureFiltering,
  RenderAntialiasing,
  RenderTextureFiltering,
} from "@/core/ipc/types/xrf-renderer";
import { IRenderChoiceOption } from "@/core/render/lib/features/render-choice-option";

const RENDER_ANTIALIASING_NAMES: Readonly<Record<RenderAntialiasing, string>> = {
  [ERenderAntialiasing.NONE]: "None",
  [ERenderAntialiasing.FXAA]: "FXAA",
  [ERenderAntialiasing.SMAA]: "SMAA",
  [ERenderAntialiasing.TAA]: "TAA",
  [ERenderAntialiasing.FSR2]: "FSR 2",
  [ERenderAntialiasing.DLSS]: "DLSS",
};

/**
 * @param mode - An antialiasing mode.
 * @returns Its name as the settings say it.
 */
export function describeRenderAntialiasing(mode: RenderAntialiasing): string {
  return RENDER_ANTIALIASING_NAMES[mode];
}

/**
 * @param mode - An antialiasing mode.
 * @returns Whether it resolves frames temporally (TAA, FSR 2, DLSS), which RCAS then sharpens at any size.
 */
export function isTemporalAntialiasing(mode: RenderAntialiasing): boolean {
  return mode === ERenderAntialiasing.TAA || mode === ERenderAntialiasing.FSR2 || mode === ERenderAntialiasing.DLSS;
}

/** Every antialiasing mode, in the order they are offered. */
export const RENDER_ANTIALIASING_OPTIONS: ReadonlyArray<IRenderChoiceOption<RenderAntialiasing>> = Object.values(
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
export const RENDER_TEXTURE_FILTERING_SHORT_OPTIONS: ReadonlyArray<IRenderChoiceOption<RenderTextureFiltering>> =
  Object.values(ERenderTextureFiltering).map((value: RenderTextureFiltering) => ({
    label: RENDER_TEXTURE_FILTERING_SHORT_NAMES[value],
    value,
  }));

/** How a level's surfaces may sample their textures, as the settings offer them. */
export const RENDER_TEXTURE_FILTERING_OPTIONS: ReadonlyArray<IRenderChoiceOption<RenderTextureFiltering>> =
  Object.values(ERenderTextureFiltering).map((value: RenderTextureFiltering) => ({
    label: RENDER_TEXTURE_FILTERING_NAMES[value],
    value,
  }));
