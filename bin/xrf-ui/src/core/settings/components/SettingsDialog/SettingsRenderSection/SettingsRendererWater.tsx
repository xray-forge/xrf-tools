import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { TRenderWaterSettings } from "@/core/render/lib/settings/render-feature-settings";
import { SettingsService } from "@/core/settings/services/settings";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { DetailSection } from "@/core/ui/layout/DetailSection";

import { useRendererOverride } from "./use-renderer-override";

/** The level's water: whether it is drawn as the engine draws it, softened in the shallows, and distorting. */
export function SettingsRendererWater(): ReactElement {
  const settingsService: SettingsService = useInjection(SettingsService);

  const water: TRenderWaterSettings = settingsService.rendererFeatures.water;

  const onSet = useRendererOverride("water");

  return (
    <DetailSection title={"Water"} description={"Water surfaces as the engine renders them."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <CheckboxFormRow
          label={"Water shading"}
          description={"Ripples and sky reflection."}
          isChecked={water.isEnabled}
          onChange={(isEnabled: boolean) => onSet({ isEnabled })}
        />

        <CheckboxFormRow
          label={"Soft water"}
          description={"Depth-based fade and shoreline foam (r2_soft_water)."}
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
