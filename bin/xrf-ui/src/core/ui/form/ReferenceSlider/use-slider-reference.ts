import { Nullable } from "@xrf/types";
import { useContext } from "react";

import {
  ISliderReference,
  ISliderReferenceSource,
  SliderReferenceContext,
} from "@/core/ui/form/ReferenceSlider/slider-reference";

/**
 * @param path - The setting's place in what the nearest source holds, if the slider has one.
 * @returns What the slider is measured by there, or none outside a source or without a place.
 */
export function useSliderReference(path: ReadonlyArray<string> | undefined): Nullable<ISliderReference> {
  const source: Nullable<ISliderReferenceSource> = useContext(SliderReferenceContext);

  if (!source || !path) {
    return null;
  }

  return { label: source.label, value: source.resolve(path) };
}
