import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { SettingsService } from "@/core/settings/services/settings";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { DetailSection } from "@/core/ui/layout/DetailSection";

/** What the renderer leaves out of a frame because none of it would show. */
export function SettingsRendererCulling(): ReactElement {
  const settingsService: SettingsService = useInjection(SettingsService);

  return (
    <DetailSection title={"Culling"} description={"Skips geometry that cannot be seen."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <CheckboxFormRow
          label={"Occlusion culling"}
          description={"Skips static geometry hidden by nearer surfaces, tested in two phases against depth."}
          isChecked={settingsService.rendererFeatures.isOcclusionCulled}
          onChange={(isOcclusionCulled: boolean) => settingsService.setRendererOverrides({ isOcclusionCulled })}
        />
      </div>
    </DetailSection>
  );
}
