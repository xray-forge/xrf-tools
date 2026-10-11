import { formatNumber } from "@xrf/format";

import {
  ERenderDebandingMode,
  ERenderDebandingQuality,
  RenderDebandingMode,
  RenderDebandingQuality,
} from "@/core/ipc/types/xrf-renderer";
import { toRenderLimits } from "@/core/render/lib/features/render-limits";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/render-feature-schema";
import { IChoiceFormRowOption } from "@/core/ui/form/ChoiceFormRow/choice-form-row-option";

/** The bounds the sky debanding's radius is offered between. */
export const RENDER_DEBANDING_LIMITS = {
  radius: toRenderLimits(RENDER_FEATURE_SCHEMA.debanding.radius, 4),
} as const;

const RENDER_DEBANDING_MODE_DESCRIPTIONS: Readonly<Record<RenderDebandingMode, string>> = {
  [ERenderDebandingMode.ENGINE]: "The game's own: the sky as drawn.",
  [ERenderDebandingMode.ENHANCED]:
    "The sky's colour bands smoothed: each sky pixel averaged with neighbours around it that differ by less than a band.",
};

/**
 * @param mode - A sky debanding mode.
 * @returns What it does, in a sentence.
 */
export function explainRenderDebandingMode(mode: RenderDebandingMode): string {
  return RENDER_DEBANDING_MODE_DESCRIPTIONS[mode];
}

const RENDER_DEBANDING_QUALITY_NAMES: Readonly<Record<RenderDebandingQuality, string>> = {
  [ERenderDebandingQuality.LOW]: "Low",
  [ERenderDebandingQuality.MEDIUM]: "Medium",
  [ERenderDebandingQuality.HIGH]: "High",
  [ERenderDebandingQuality.ULTRA]: "Ultra",
};

/**
 * @param quality - A sky debanding quality.
 * @returns Its name as the settings say it.
 */
export function describeRenderDebandingQuality(quality: RenderDebandingQuality): string {
  return RENDER_DEBANDING_QUALITY_NAMES[quality];
}

/** The sky debanding qualities, in the order they are offered. */
export const RENDER_DEBANDING_QUALITY_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderDebandingQuality>> =
  Object.values(ERenderDebandingQuality).map((value: RenderDebandingQuality) => ({
    label: describeRenderDebandingQuality(value),
    value,
  }));

/** @returns How far out a sky pixel's neighbours are read at most. */
export function formatDebandingRadius(radius: number): string {
  return `${formatNumber(radius, 0)} px`;
}
