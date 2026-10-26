import { sliderClasses, Theme } from "@mui/material";
import { SystemStyleObject } from "@mui/system";
import { saturate } from "@xrf/math";
import { Nullable } from "@xrf/types";

import { SLIDER_REFERENCE } from "@/core/theme/tokens";

/**
 * @param recommended - The range xrf recommends, none where the setting is not rated.
 * @param min - The slider's least value.
 * @param max - The slider's most value.
 * @param wastedAbove - The value past which a higher one only costs more, none where not measured.
 * @returns A slider's reference notch standing out of its track, the recommended range as a band on a rail taller
 *   than the track, the rail neutral outside it, and red past the useful end: the warning is the slider's own colour,
 *   never the amber accent's. All fade as the rest of the slider does while it is disabled.
 */
export function getSliderReferenceSx(
  recommended: Nullable<readonly [number, number]> | undefined,
  min: number,
  max: number,
  wastedAbove?: Nullable<number>
): (theme: Theme) => SystemStyleObject<Theme> {
  return (theme: Theme): SystemStyleObject<Theme> => {
    const palette = (theme.vars ?? theme).palette;
    const notch: SystemStyleObject<Theme> = {
      [`& .${sliderClasses.mark}`]: {
        backgroundColor: palette.text.secondary,
        height: SLIDER_REFERENCE.notchHeight,
        width: SLIDER_REFERENCE.notchWidth,
        borderRadius: `${SLIDER_REFERENCE.notchWidth / 2}px`,
        opacity: 1,
      },
      [`& .${sliderClasses.markActive}`]: { backgroundColor: palette.text.secondary, opacity: 1 },
      [`&.${sliderClasses.disabled} .${sliderClasses.mark}`]: { opacity: theme.palette.action.disabledOpacity },
    };

    const isWasted: boolean = wastedAbove !== null && wastedAbove !== undefined && wastedAbove < max;

    if ((!recommended && !isWasted) || max <= min) {
      return notch;
    }

    const outside: string = `color-mix(in srgb, ${palette.text.primary} ${SLIDER_REFERENCE.railShare * 100}%, transparent)`;
    const inside: string = `color-mix(in srgb, ${palette.success.main} ${SLIDER_REFERENCE.bandShare * 100}%, transparent)`;
    const wasted: string = `color-mix(in srgb, ${palette.error.main} ${SLIDER_REFERENCE.wastedShare * 100}%, transparent)`;
    const low: string = toTrackShare(recommended?.[0] ?? min, min, max);
    // The band ends where the wasted end starts, should the two meet.
    const high: string = toTrackShare(Math.min(recommended?.[1] ?? min, wastedAbove ?? max), min, max);
    const end: string = isWasted ? toTrackShare(wastedAbove as number, min, max) : "100%";
    const band: string = recommended ? `${inside} ${low} ${high}, ` : "";

    return {
      ...notch,
      [`& .${sliderClasses.rail}`]: {
        opacity: 1,
        height: SLIDER_REFERENCE.railHeight,
        borderRadius: `${SLIDER_REFERENCE.railHeight / 2}px`,
        background:
          `linear-gradient(to right, ${outside} 0 ${low}, ${band}${outside} ${high} ${end}` +
          (isWasted ? `, ${wasted} ${end} 100%)` : ")"),
      },
      [`&.${sliderClasses.disabled} .${sliderClasses.rail}`]: { opacity: theme.palette.action.disabledOpacity },
    };
  };
}

/** Where a value stands along a track from `min` to `max`, as a CSS percentage held to the track. */
function toTrackShare(at: number, min: number, max: number): string {
  return `${(saturate((at - min) / (max - min)) * 100).toFixed(2)}%`;
}
