import { Stack, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement, useState } from "react";

import { renderCommands } from "@/core/ipc/commands/render";
import { RenderLibraries } from "@/core/ipc/types/xrf-renderer";
import { RenderSettingsService } from "@/core/render/services/render-settings";
import { DetailSection } from "@/core/ui/layout/DetailSection";
import { Logger, useLogger } from "@/lib/logging";
import { useMountEffect } from "@/lib/react";

import { IAboutRow } from "./about-row";
import { SettingsAboutRow } from "./SettingsAboutRow";
import { describeGraphics, DLSS_ATTRIBUTION } from "./SettingsGraphicsSection.utils";

/**
 * The GPU each graphics API would use, the API in use, and the DLSS version with the attribution NVIDIA's licence
 * requires.
 */
export function SettingsGraphicsSection(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const [libraries, setLibraries] = useState<Nullable<RenderLibraries>>(null);

  // The DLSS version cannot change while the application runs; the backends are probed again, as in Display.
  useMountEffect(() => {
    void renderSettingsService.refreshRenderBackends();

    renderCommands
      .describeLibraries()
      .then((it: RenderLibraries) => setLibraries(it))
      .catch((error: unknown) => log.error("Failed to read the graphics libraries:", error));
  });

  const rows: Array<IAboutRow> = describeGraphics(
    renderSettingsService.renderBackend,
    renderSettingsService.renderGpu,
    renderSettingsService.renderBackends,
    libraries
  );

  return (
    <DetailSection
      data-testid={"settings-graphics-section"}
      title={"Graphics"}
      description={"The GPU and driver each graphics API would use, and the DLSS version."}
    >
      <Stack className={"mt-2"} spacing={0.5}>
        {rows.length === 0 ? (
          <Typography variant={"caption"}>Reading graphics details...</Typography>
        ) : (
          rows.map((it: IAboutRow) => <SettingsAboutRow key={it.label} label={it.label} value={it.value} />)
        )}

        {libraries?.dlss ? (
          <Typography className={"pt-2 opacity-70"} variant={"caption"}>
            {DLSS_ATTRIBUTION}
          </Typography>
        ) : null}
      </Stack>
    </DetailSection>
  );
}
