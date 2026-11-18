import { useInjection } from "@wirestate/react";
import { RenderLibraries } from "@xrf/ipc/types/xrf-renderer";
import { useMountEffect } from "@xrf/react/mount-effect";
import { Nullable } from "@xrf/types";
import { EditorPanelProperty } from "@xrf/ui/editor/panel";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { Stack } from "@xrf/ui/layout/stack";
import { Typography } from "@xrf/ui/typography";
import { ReactElement } from "react";

import { RenderSettingsService } from "@/core/render/services/render-settings.service";
import { SettingsAboutService } from "@/core/settings-dialog/services/settings-about";

import { IAboutRow } from "./about-row";
import { SettingsAboutPending } from "./SettingsAboutPending";
import { describeGraphics, DLSS_ATTRIBUTION } from "./SettingsGraphicsSection.utils";

/**
 * The GPU each graphics API would use, the API in use, and the DLSS version with the attribution NVIDIA's licence
 * requires.
 */
export function SettingsGraphicsSection(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);
  const aboutService: SettingsAboutService = useInjection(SettingsAboutService);

  const libraries: Nullable<RenderLibraries> = aboutService.libraries.value;
  const rows: Array<IAboutRow> = describeGraphics(
    renderSettingsService.renderBackend,
    renderSettingsService.renderGpu,
    renderSettingsService.renderBackends,
    libraries
  );

  // The backends are probed again, as in Display; the libraries are read once, with the rest of the section.
  useMountEffect(() => {
    void renderSettingsService.refreshRenderBackends();
  });

  return (
    <DetailSection
      data-testid={"settings-graphics-section"}
      title={"Graphics"}
      description={"The GPU and driver each graphics API would use, and the DLSS version."}
    >
      <Stack className={"mt-2"} spacing={0.5}>
        {rows.length === 0 ? (
          <SettingsAboutPending error={aboutService.libraries.error} label={"Reading graphics details..."} />
        ) : (
          rows.map((it: IAboutRow) => (
            <EditorPanelProperty key={it.label} label={it.label} value={it.value} isMonospace isInline />
          ))
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
