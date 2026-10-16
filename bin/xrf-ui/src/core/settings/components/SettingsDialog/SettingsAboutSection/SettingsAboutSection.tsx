import { ReactElement } from "react";

import { cn } from "@/core/theme/class-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

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
