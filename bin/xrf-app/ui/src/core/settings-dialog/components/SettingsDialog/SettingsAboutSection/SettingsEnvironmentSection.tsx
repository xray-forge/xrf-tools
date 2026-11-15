import { useInjection } from "@wirestate/react";
import { HostInfo } from "@xrf/ipc/types/xrf-app-system";
import { Nullable } from "@xrf/types";
import { EditorPanelProperty } from "@xrf/ui/editor/panel";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { Stack } from "@xrf/ui/layout/stack";
import { Typography } from "@xrf/ui/typography";
import { ReactElement } from "react";

import { SettingsAboutService } from "@/core/settings-dialog/services/settings-about";

import { IAboutRow } from "./about-row";
import { describeHost } from "./SettingsEnvironmentSection.utils";

/**
 * The machine and the runtime the application found when it started.
 */
export function SettingsEnvironmentSection(): ReactElement {
  const aboutService: SettingsAboutService = useInjection(SettingsAboutService);

  const host: Nullable<HostInfo> = aboutService.host.value;

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
