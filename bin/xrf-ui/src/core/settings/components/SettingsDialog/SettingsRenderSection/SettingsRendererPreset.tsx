import { Button, ToggleButton, ToggleButtonGroup } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { isRenderFeatureChoiceCustom } from "@/core/render/lib/settings/render-feature-choice";
import { ERenderPreset } from "@/core/render/lib/settings/render-preset";
import { RenderSettingsService } from "@/core/render/services/render-settings";
import { DetailSection } from "@/core/ui/layout/DetailSection";

const PRESET_LABELS: Record<ERenderPreset, string> = {
  [ERenderPreset.BASE]: "Base",
  [ERenderPreset.EDITING]: "Editing",
};

/** Which preset the renderer's features follow, whether anything is changed on top of it, and the way back. */
export function SettingsRendererPreset(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const { preset } = renderSettingsService.rendererChoice;
  const isCustom: boolean = isRenderFeatureChoiceCustom(renderSettingsService.rendererChoice);

  return (
    <DetailSection
      title={"Preset"}
      description={
        "Applies to every tab except Display. Base matches the game's defaults; Editing favours responsiveness."
      }
      fact={isCustom ? `Custom, from ${PRESET_LABELS[preset]}` : PRESET_LABELS[preset]}
      action={
        isCustom ? (
          <Button size={"small"} onClick={() => renderSettingsService.setRendererPreset(preset)}>
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
        onChange={(_, next: Nullable<ERenderPreset>) => {
          if (next !== null) {
            renderSettingsService.setRendererPreset(next);
          }
        }}
      >
        {Object.values(ERenderPreset).map((value: ERenderPreset) => (
          <ToggleButton key={value} value={value}>
            {PRESET_LABELS[value]}
          </ToggleButton>
        ))}
      </ToggleButtonGroup>
    </DetailSection>
  );
}
