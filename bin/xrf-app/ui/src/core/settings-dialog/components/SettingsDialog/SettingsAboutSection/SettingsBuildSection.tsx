import { useInjection } from "@wirestate/react";
import { BuildInfo } from "@xrf/ipc/types/xrf-build-info";
import { Nullable } from "@xrf/types";
import { EditorPanelProperty } from "@xrf/ui/editor/panel";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { Stack } from "@xrf/ui/layout/stack";
import { Typography } from "@xrf/ui/typography";
import { ReactElement } from "react";

import { SettingsAboutService } from "@/core/settings-dialog/services/settings-about";

import { IAboutRow } from "./about-row";
import { describeBuild } from "./SettingsBuildSection.utils";

/**
 * Which build of the application is running, and where it came from.
 */
export function SettingsBuildSection(): ReactElement {
  const aboutService: SettingsAboutService = useInjection(SettingsAboutService);

  const build: Nullable<BuildInfo> = aboutService.build.value;

  return (
    <DetailSection
      data-testid={"settings-build-section"}
      title={"Build"}
      description={"Which build of the application is running, and where it came from."}
    >
      <Stack className={"mt-2"} spacing={0.5}>
        {build ? (
          describeBuild(build).map((it: IAboutRow) => (
            <EditorPanelProperty key={it.label} label={it.label} value={it.value} href={it.href} isMonospace isInline />
          ))
        ) : (
          <Typography variant={"caption"}>Reading build details...</Typography>
        )}
      </Stack>
    </DetailSection>
  );
}
