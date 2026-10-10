import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { RenderSettingsService } from "@/core/render/services/render-settings";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { DetailSection } from "@/core/ui/layout/DetailSection";

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
      </div>
    </DetailSection>
  );
}
