import { systemCommands } from "@xrf/ipc/commands/system";
import { HostInfo } from "@xrf/ipc/types/xrf-app-system";
import { Logger } from "@xrf/logging";
import { useLogger } from "@xrf/logging/react";
import { useMountEffect } from "@xrf/react/mount-effect";
import { Nullable } from "@xrf/types";
import { EditorPanelProperty } from "@xrf/ui/editor/panel";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { Stack } from "@xrf/ui/layout/stack";
import { Typography } from "@xrf/ui/typography";
import { ReactElement, useState } from "react";

import { IAboutRow } from "./about-row";
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
            <EditorPanelProperty key={it.label} label={it.label} value={it.value} href={it.href} isMonospace isInline />
          ))
        ) : (
          <Typography variant={"caption"}>Reading environment details...</Typography>
        )}
      </Stack>
    </DetailSection>
  );
}
