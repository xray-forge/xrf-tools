import { sliderClasses, Theme } from "@mui/material";
import { SystemStyleObject } from "@mui/system";
import { saturate } from "@xrf/math";
import { Nullable } from "@xrf/types";

import { SLIDER_REFERENCE } from "@/core/theme/tokens";

/**
 * @param recommended - The range xrf recommends, none where the setting is not rated.
 * @param min - The slider's least value.
 * @param max - The slider's most value.
 * @returns A slider's reference notch standing out of its track, and the recommended range as a band on a rail taller
 *   than the track, the rail neutral outside it: the warning is the slider's own colour, never the amber accent's.
 *   Both fade as the rest of the slider does while it is disabled.
 */
export function getSliderReferenceSx(
  recommended: Nullable<readonly [number, number]> | undefined,
  min: number,
  max: number
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

    if (!recommended || max <= min) {
      return notch;
    }

    const outside: string = `color-mix(in srgb, ${palette.text.primary} ${SLIDER_REFERENCE.railShare * 100}%, transparent)`;
    const inside: string = `color-mix(in srgb, ${palette.success.main} ${SLIDER_REFERENCE.bandShare * 100}%, transparent)`;
    const low: string = toTrackShare(recommended[0], min, max);
    const high: string = toTrackShare(recommended[1], min, max);

    return {
      ...notch,
      [`& .${sliderClasses.rail}`]: {
        opacity: 1,
        height: SLIDER_REFERENCE.railHeight,
        borderRadius: `${SLIDER_REFERENCE.railHeight / 2}px`,
        background: `linear-gradient(to right, ${outside} 0 ${low}, ${inside} ${low} ${high}, ${outside} ${high} 100%)`,
      },
      [`&.${sliderClasses.disabled} .${sliderClasses.rail}`]: { opacity: theme.palette.action.disabledOpacity },
    };
  };
}

/** Where a value stands along a track from `min` to `max`, as a CSS percentage held to the track. */
function toTrackShare(at: number, min: number, max: number): string {
  return `${(saturate((at - min) / (max - min)) * 100).toFixed(2)}%`;
}
