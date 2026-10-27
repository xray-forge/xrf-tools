import { useInjection } from "@wirestate/react";
import { CheckboxFormRow } from "@xrf/ui/form/CheckboxFormRow";
import { DetailSection } from "@xrf/ui/layout/DetailSection";
import { ReactElement } from "react";

import { TRenderWaterSettings } from "@/core/render/lib/settings/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/** The level's water: whether it is drawn as the engine draws it, softened in the shallows, and distorting. */
export function SettingsRendererWater(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const water: TRenderWaterSettings = renderSettingsService.rendererFeatures.water;

  const onSet = useRendererOverride("water");

  return (
    <DetailSection title={"Water"} description={"Water surfaces as the engine renders them."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <CheckboxFormRow
          label={"Water"}
          description={"Ripples and sky reflection."}
          isChecked={water.isEnabled}
          onChange={(isEnabled: boolean) => onSet({ isEnabled })}
        />

        <CheckboxFormRow
          label={"Soft water"}
          description={"Fades the water by its depth and lays foam in the shallows. Game setting: r2_soft_water."}
          isChecked={water.isSoft}
          isDisabled={!water.isEnabled}
          onChange={(isSoft: boolean) => onSet({ isSoft })}
        />

        <CheckboxFormRow
          label={"Distortion"}
          description={"Refracts what is seen through the surface."}
          isChecked={water.isDistorted}
          isDisabled={!water.isEnabled}
          onChange={(isDistorted: boolean) => onSet({ isDistorted })}
        />
      </div>
    </DetailSection>
  );
}
