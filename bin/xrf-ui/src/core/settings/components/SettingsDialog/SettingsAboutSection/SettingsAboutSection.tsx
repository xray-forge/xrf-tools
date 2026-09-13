import { Box } from "@mui/material";
import { ReactElement } from "react";

import { BaseComponentProps } from "@/lib/dom/element-types";

import { SettingsBuildSection } from "./SettingsBuildSection";
import { SettingsEnvironmentSection } from "./SettingsEnvironmentSection";
import { SettingsRuntimeSection } from "./SettingsRuntimeSection";
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
    <Box
      data-testid={dataTestId}
      className={className}
      id={id}
      sx={{ display: "flex", flexDirection: "column", gap: 3 }}
    >
      <SettingsBuildSection />
      <SettingsRuntimeSection />
      <SettingsWebviewSection />
      <SettingsEnvironmentSection />
    </Box>
  );
}
