import { formatNumber, formatPercent } from "@xrf/format";

import { toRenderLimits } from "@/core/render/lib/features/render-limits";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/render-feature-schema";

/** The bounds each contact shadow value is offered between. */
export const RENDER_CONTACT_SHADOW_LIMITS = {
  intensity: toRenderLimits(RENDER_FEATURE_SCHEMA.shadows.contact.intensity, 0.05),
  length: toRenderLimits(RENDER_FEATURE_SCHEMA.shadows.contact.length, 0.05),
  lights: toRenderLimits(RENDER_FEATURE_SCHEMA.shadows.contact.lights, 1),
  steps: toRenderLimits(RENDER_FEATURE_SCHEMA.shadows.contact.steps, 1),
  thickness: toRenderLimits(RENDER_FEATURE_SCHEMA.shadows.contact.thickness, 0.01),
} as const;

/** @returns How far a contact shadow's ray reaches towards the sun or a light, in metres. */
export function formatContactShadowLength(length: number): string {
  return `${formatNumber(length, 2)} m`;
}

/** @returns How much light a contact shadow takes away. */
export function formatContactShadowIntensity(intensity: number): string {
  return formatPercent(intensity);
}

/** @returns How deep what the depth shows is taken to be, in centimetres. */
export function formatContactShadowThickness(thickness: number): string {
  return `${formatNumber(thickness * 100, 0)} cm`;
}

/** @returns How many depth reads a contact shadow's ray takes. */
export function formatContactShadowSteps(steps: number): string {
  return formatNumber(steps, 0);
}

/**
 * @param lights - Local lights each pixel marches contact shadows towards at most, the strongest there.
 * @returns It as the popover reads it: a count of lights, or the sun's alone for none.
 */
export function formatContactShadowLights(lights: number): string {
  return lights ? `${lights} light${lights > 1 ? "s" : ""}` : "Sun only";
}
