import { Stack, Typography } from "@mui/material";
import { Nullable } from "@xrf/types";
import { ReactElement, useState } from "react";

import { systemCommands } from "@/core/ipc/commands/system";
import { HostInfo } from "@/core/ipc/types/xrf-app";
import { DetailSection } from "@/core/ui/layout/DetailSection";
import { Logger, useLogger } from "@/lib/logging";
import { useMountEffect } from "@/lib/react";

import { IAboutRow } from "./about-row";
import { SettingsAboutRow } from "./SettingsAboutRow";
import { describeHost } from "./SettingsEnvironmentSection.utils";

/**
 * The machine and the runtime the application found when it started.
 */
export function SettingsEnvironmentSection(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const [host, setHost] = useState<Nullable<HostInfo>>(null);

  // Read once: none of it changes while the application runs, which is why it is a separate command from the usage.
  useMountEffect(() => {
    systemCommands
      .getHostInfo()
      .then((it: HostInfo) => setHost(it))
      .catch((error: unknown) => log.error("Failed to read host info:", error));
  });

  return (
    <DetailSection
      data-testid={"settings-environment-section"}
      title={"Environment"}
      description={"The machine and the runtime the application found when it started."}
    >
      <Stack className={"mt-2"} spacing={0.5}>
        {host ? (
          describeHost(host).map((it: IAboutRow) => (
            <SettingsAboutRow key={it.label} label={it.label} value={it.value} href={it.href} />
          ))
        ) : (
          <Typography variant={"caption"}>Reading environment details...</Typography>
        )}
      </Stack>
    </DetailSection>
  );
}
