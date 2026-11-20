import { useInjection } from "@wirestate/react";
import { useMountEffect } from "@xrf/react/mount-effect";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { cn } from "@xrf/ui/theme";
import { ReactElement } from "react";

import { SettingsAboutService } from "@/core/settings-dialog/services/settings-about";

import { SettingsBuildSection } from "./SettingsBuildSection";
import { SettingsEnvironmentSection } from "./SettingsEnvironmentSection";
import { SettingsGraphicsSection } from "./SettingsGraphicsSection";
import { SettingsRuntimeSection } from "./SettingsRuntimeSection";
import { SettingsVariablesSection } from "./SettingsVariablesSection";
import { SettingsWebviewSection } from "./SettingsWebviewSection";

/**
 * Everything this window can say about the instance it belongs to.
 */
export function SettingsAboutSection({
  "data-testid": dataTestId = "settings-about-section",
  className,
  id,
}: BaseComponentProps): ReactElement {
  const aboutService: SettingsAboutService = useInjection(SettingsAboutService);

  useMountEffect(() => {
    void aboutService.read();
  });

  return (
    <div data-testid={dataTestId} id={id} className={cn("flex flex-col gap-6", className)}>
      <SettingsBuildSection />
      <SettingsRuntimeSection />
      <SettingsWebviewSection />
      <SettingsEnvironmentSection />
      <SettingsVariablesSection />
      <SettingsGraphicsSection />
    </div>
  );
}
