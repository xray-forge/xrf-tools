import { formatPercent } from "@xrf/format";
import { IChoiceFormRowOption } from "@xrf/ui/form/ChoiceFormRow/choice-form-row-option";

import { IRenderLimits } from "@/core/render/lib/features/render-limits";
import { FRAME_RATE_LIMITS, TFrameRateLimit } from "@/core/render/lib/settings/render-frame-rate-limit";
import {
  ERenderResolution,
  RENDER_OUTPUT_SCALES,
  RENDER_RESOLUTIONS,
} from "@/core/render/lib/settings/render-resolution";

/** The frame rate limits, in the order they are offered. */
export const RENDER_FRAME_RATE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<TFrameRateLimit>> = FRAME_RATE_LIMITS.map(
  (value: TFrameRateLimit) => ({ label: value === "none" ? "No cap" : `${value} fps`, value })
);

/**
 * @param resolution - A resolution every viewport may draw at.
 * @returns Its name as the settings say it.
 */
function describeRenderResolution(resolution: ERenderResolution): string {
  switch (resolution) {
    case ERenderResolution.AUTO:
      return "Auto";
    case ERenderResolution.HEIGHT_720:
      return "720p";
    case ERenderResolution.HEIGHT_1080:
      return "1080p";
    case ERenderResolution.HEIGHT_1440:
      return "1440p";
    case ERenderResolution.HEIGHT_2160:
      return "4K";
  }
}

/** The bounds the automatic resolution's scale is offered between, five percent a step. */
export const RENDER_OUTPUT_SCALE_LIMITS: IRenderLimits = {
  min: RENDER_OUTPUT_SCALES.min,
  max: RENDER_OUTPUT_SCALES.max,
  step: 0.05,
};

/** @returns The automatic resolution's scale, as a share of the viewport's sides. */
export function formatOutputScale(scale: number): string {
  return formatPercent(scale);
}

/** The resolutions, in the order they are offered. */
export const RENDER_RESOLUTION_OPTIONS: ReadonlyArray<IChoiceFormRowOption<ERenderResolution>> = RENDER_RESOLUTIONS.map(
  (value: ERenderResolution) => ({ label: describeRenderResolution(value), value })
);
