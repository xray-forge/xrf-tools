import { Button } from "@mui/material";
import { useInjection } from "@wirestate/react";
import {
  ERendererAntialiasing,
  ERendererPreset,
  IRendererFeatureSettings,
  isRendererFeatureChoiceCustom,
} from "@xrf/renderer";
import { ReactElement } from "react";

import { describeRenderAntialiasing } from "@/core/render/lib/features";
import { SettingsService } from "@/core/settings/services/settings";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { ChoiceFormRow, IChoiceFormRowOption } from "@/core/ui/form/ChoiceFormRow";
import { DetailSection } from "@/core/ui/layout/DetailSection";

const PRESET_LABELS: Record<ERendererPreset, string> = {
  [ERendererPreset.BASE]: "Base",
  [ERendererPreset.EDITING]: "Editing",
};

const PRESET_OPTIONS: ReadonlyArray<IChoiceFormRowOption<ERendererPreset>> = Object.values(ERendererPreset).map(
  (value: ERendererPreset) => ({ label: PRESET_LABELS[value], value })
);

const ANTIALIASING_OPTIONS: ReadonlyArray<IChoiceFormRowOption<ERendererAntialiasing>> = Object.values(
  ERendererAntialiasing
).map((value: ERendererAntialiasing) => ({ label: describeRenderAntialiasing(value), value }));

/** Which preset the renderer's features follow, and the features that are not a level's alone. */
export function SettingsRendererFeatures(): ReactElement {
  const settingsService: SettingsService = useInjection(SettingsService);

  const { preset } = settingsService.rendererChoice;
  const features: IRendererFeatureSettings = settingsService.rendererFeatures;
  const isCustom: boolean = isRendererFeatureChoiceCustom(settingsService.rendererChoice);

  return (
    <DetailSection
      title={"Features"}
      description={"What every viewport draws with: a preset, and whatever is changed on top of it."}
      fact={isCustom ? `Custom, from ${PRESET_LABELS[preset]}` : PRESET_LABELS[preset]}
      action={
        isCustom ? (
          <Button size={"small"} onClick={() => settingsService.setRendererPreset(preset)}>
            Back to {PRESET_LABELS[preset]}
          </Button>
        ) : null
      }
    >
      <div className={"mt-4 flex flex-col gap-6"}>
        <ChoiceFormRow
          label={"Preset"}
          description={"Base is the game's own defaults. Editing puts responsiveness before looks."}
          options={PRESET_OPTIONS}
          value={preset}
          onChange={settingsService.setRendererPreset}
        />

        <ChoiceFormRow
          label={"Anti-aliasing"}
          description={"How the frame's edges are smoothed. SMAA is crisp and stable; FXAA is cheaper and softer."}
          options={ANTIALIASING_OPTIONS}
          value={features.antialiasing}
          onChange={(antialiasing: ERendererAntialiasing) => settingsService.setRendererOverrides({ antialiasing })}
        />

        <CheckboxFormRow
          label={"GPU timings"}
          description={"Times every pass on the GPU for the readout."}
          isChecked={features.isGpuTimed}
          onChange={(isGpuTimed: boolean) => settingsService.setRendererOverrides({ isGpuTimed })}
        />
      </div>
    </DetailSection>
  );
}
