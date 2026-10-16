import { ERenderParallaxMode, RenderParallaxMode } from "@/core/ipc/types/xrf-renderer";
import { IChoiceFormRowOption } from "@/core/ui/form/ChoiceFormRow/choice-form-row-option";

/** The parallax modes, in the order they are offered. */
export const RENDER_PARALLAX_MODE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderParallaxMode>> = [
  { label: "Engine", value: ERenderParallaxMode.ENGINE },
  { label: "Enhanced", value: ERenderParallaxMode.ENHANCED },
];
