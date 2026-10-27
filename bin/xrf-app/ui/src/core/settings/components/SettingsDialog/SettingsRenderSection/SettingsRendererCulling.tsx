import { useInjection } from "@wirestate/react";
import { CheckboxFormRow } from "@xrf/ui/form/CheckboxFormRow";
import { DetailSection } from "@xrf/ui/layout/DetailSection";
import { ReactElement } from "react";

import { RenderSettingsService } from "@/core/render/services/render-settings.service";

/** What the renderer leaves out of a frame because none of it would show. */
export function SettingsRendererCulling(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  return (
    <DetailSection title={"Culling"} description={"Skips geometry that cannot be seen."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <CheckboxFormRow
          label={"Occlusion culling"}
          description={"Skips static geometry hidden by nearer surfaces, tested in two phases against depth."}
          isChecked={renderSettingsService.rendererFeatures.isOcclusionCulled}
          onChange={(isOcclusionCulled: boolean) => renderSettingsService.setRendererOverrides({ isOcclusionCulled })}
        />
        <CheckboxFormRow
          label={"Opaque depth pass"}
          description={
            "Draws the opaque surfaces' depth first, so each pixel is shaded once where surfaces stack; costs a little " +
            "over open ground seen from above."
          }
          isChecked={renderSettingsService.rendererFeatures.isDepthPrepassed}
          onChange={(isDepthPrepassed: boolean) => renderSettingsService.setRendererOverrides({ isDepthPrepassed })}
        />
      </div>
    </DetailSection>
  );
}
