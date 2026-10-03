import { Button, ToggleButton, ToggleButtonGroup } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { isRendererFeatureChoiceCustom } from "@/core/render/lib/contract/renderer-feature-choice";
import { ERendererPreset } from "@/core/render/lib/contract/renderer-preset";
import { SettingsService } from "@/core/settings/services/settings";
import { DetailSection } from "@/core/ui/layout/DetailSection";

const PRESET_LABELS: Record<ERendererPreset, string> = {
  [ERendererPreset.BASE]: "Base",
  [ERendererPreset.EDITING]: "Editing",
};

/** Which preset the renderer's features follow, whether anything is changed on top of it, and the way back. */
export function SettingsRendererPreset(): ReactElement {
  const settingsService: SettingsService = useInjection(SettingsService);

  const { preset } = settingsService.rendererChoice;
  const isCustom: boolean = isRendererFeatureChoiceCustom(settingsService.rendererChoice);

  return (
    <DetailSection
      title={"Preset"}
      description={
        "Applies to every tab except Display. Base matches the game's defaults; Editing favours responsiveness."
      }
      fact={isCustom ? `Custom, from ${PRESET_LABELS[preset]}` : PRESET_LABELS[preset]}
      action={
        isCustom ? (
          <Button size={"small"} onClick={() => settingsService.setRendererPreset(preset)}>
            Back to {PRESET_LABELS[preset]}
          </Button>
        ) : null
      }
    >
      <ToggleButtonGroup
        aria-label={"Preset"}
        exclusive
        color={"primary"}
        size={"small"}
        value={preset}
        onChange={(_, next: Nullable<ERendererPreset>) => {
          if (next !== null) {
            settingsService.setRendererPreset(next);
          }
        }}
      >
        {Object.values(ERendererPreset).map((value: ERendererPreset) => (
          <ToggleButton key={value} value={value}>
            {PRESET_LABELS[value]}
          </ToggleButton>
        ))}
      </ToggleButtonGroup>
    </DetailSection>
  );
}
