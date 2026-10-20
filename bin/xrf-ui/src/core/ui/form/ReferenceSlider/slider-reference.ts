import { Nullable } from "@xrf/types";
import { createContext, useContext } from "react";

/** Where a slider's value stands against what it is measured by. */
export interface ISliderReference {
  /** What the slider returns to on a reset: the settings' value in a popover, the preset's in the settings. */
  value: Nullable<number>;
  /** Names where that value comes from, such as "Settings" or "Base preset". */
  label: string;
}

/** What the sliders within read their reference from: the source's name, and its number at a setting's place. */
export interface ISliderReferenceSource {
  label: string;
  resolve: (path: ReadonlyArray<string>) => Nullable<number>;
}

/** The source the sliders within measure themselves by; none outside a provider, where they show no reference. */
export const SliderReferenceContext = createContext<Nullable<ISliderReferenceSource>>(null);

/** What a value outside the recommended range says, below it and above it. */
export const SLIDER_BELOW_RECOMMENDED: string = "Below the recommended range: visible misses or artefacts";
export const SLIDER_ABOVE_RECOMMENDED: string = "Past the recommended range: costs more for little visible gain";

/**
 * @param path - The setting's place in what the nearest source holds, if the slider has one.
 * @returns What the slider is measured by there, or none outside a source or without a place.
 */
export function useSliderReference(path: ReadonlyArray<string> | undefined): Nullable<ISliderReference> {
  const source: Nullable<ISliderReferenceSource> = useContext(SliderReferenceContext);

  return source && path ? { label: source.label, value: source.resolve(path) } : null;
}

/**
 * @param value - A slider's value.
 * @param recommended - The range xrf recommends, if it rates the setting.
 * @returns What a value outside the range says, or none within it.
 */
export function describeSliderRecommendation(
  value: number,
  recommended: Nullable<readonly [number, number]> | undefined
): Nullable<string> {
  if (!recommended) {
    return null;
  }

  if (value < recommended[0]) {
    return SLIDER_BELOW_RECOMMENDED;
  }

  return value > recommended[1] ? SLIDER_ABOVE_RECOMMENDED : null;
}
