import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { SettingsService } from "@/core/settings/services/settings";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { DetailSection } from "@/core/ui/layout/DetailSection";

import { useRendererOverride } from "./use-renderer-override";

/** How the level's surfaces are displaced by their height. */
export function SettingsRendererParallax(): ReactElement {
  const settingsService: SettingsService = useInjection(SettingsService);
  const onSet = useRendererOverride("parallax");

  return (
    <DetailSection title={"Parallax"} description={"Displaces surfaces by their height."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <CheckboxFormRow
          label={"Steep parallax"}
          description={"The engine's steep parallax within 12 m on level surfaces whose texture asks for it."}
          isChecked={settingsService.rendererFeatures.parallax.isSteep}
          onChange={(isSteep: boolean) => onSet({ isSteep })}
        />
      </div>
    </DetailSection>
  );
}
