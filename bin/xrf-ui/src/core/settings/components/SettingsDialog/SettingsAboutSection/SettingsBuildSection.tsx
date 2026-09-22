import { Stack, Typography } from "@mui/material";
import { Nullable } from "@xrf/types";
import { ReactElement, useState } from "react";

import { systemCommands } from "@/core/ipc/commands/system";
import { BuildInfo } from "@/core/ipc/types/xrf-build-info";
import { DetailSection } from "@/core/ui/layout/DetailSection";
import { Logger, useLogger } from "@/lib/logging";
import { useMountEffect } from "@/lib/react";

import { IAboutRow } from "./about-row";
import { SettingsAboutRow } from "./SettingsAboutRow";
import { describeBuild } from "./SettingsBuildSection.utils";

/**
 * Which build of the application is running, and where it came from.
 */
export function SettingsBuildSection(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const [build, setBuild] = useState<Nullable<BuildInfo>>(null);

  // Read once: a running application cannot become a different build.
  useMountEffect(() => {
    systemCommands
      .getBuildInfo()
      .then((it: BuildInfo) => setBuild(it))
      .catch((error: unknown) => log.error("Failed to read build info:", error));
  });

  return (
    <DetailSection
      data-testid={"settings-build-section"}
      title={"Build"}
      description={"Which build of the application is running, and where it came from."}
    >
      <Stack className={"mt-2"} spacing={0.5}>
        {build ? (
          describeBuild(build).map((it: IAboutRow) => (
            <SettingsAboutRow key={it.label} label={it.label} value={it.value} href={it.href} />
          ))
        ) : (
          <Typography variant={"caption"}>Reading build details...</Typography>
        )}
      </Stack>
    </DetailSection>
  );
}
