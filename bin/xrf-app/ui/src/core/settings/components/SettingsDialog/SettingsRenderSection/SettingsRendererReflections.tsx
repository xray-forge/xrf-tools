import { useInjection } from "@wirestate/react";
import { ERenderReflectionMode } from "@xrf/ipc/types/xrf-renderer";
import { CheckboxFormRow } from "@xrf/ui/form/checkbox-row";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { ReactElement } from "react";

import { RenderSettingRows } from "@/core/render/components/settings/RenderSettingRows";
import {
  explainRenderReflectionMode,
  RENDER_REFLECTION_SETTINGS,
} from "@/core/render/lib/features/tracing/render-reflection-choices";
import {
  IRenderFeatureSettings,
  TRenderReflectionSettings,
} from "@/core/render/lib/settings/choice/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/** What the ground, puddles and wet and glossy surfaces reflect: the sky's cube alone, or traced over the screen. */
export function SettingsRendererReflections(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const features: IRenderFeatureSettings = renderSettingsService.rendererFeatures;
  const reflections: TRenderReflectionSettings = features.reflections;

  const onSet = useRendererOverride("reflections");

  return (
    <DetailSection title={"Reflections"} description={"The ground, puddles, and wet and glossy surfaces."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <CheckboxFormRow
          label={RENDER_REFLECTION_SETTINGS.mode.label}
          description={explainRenderReflectionMode(reflections.mode)}
          isChecked={reflections.mode === ERenderReflectionMode.ENHANCED}
          onChange={(isOn: boolean) =>
            onSet({ mode: isOn ? ERenderReflectionMode.ENHANCED : ERenderReflectionMode.ENGINE })
          }
        />

        <RenderSettingRows
          settings={RENDER_REFLECTION_SETTINGS}
          names={[
            "traceSize",
            "steps",
            "probe",
            "intensity",
            "distance",
            "reach",
            "skyGain",
            "traceRoughness",
            "isDryTraced",
            "isRayBudgeted",
            "isGrassReflected",
          ]}
          group={reflections}
          features={features}
          onSet={onSet}
        />
      </div>
    </DetailSection>
  );
}
