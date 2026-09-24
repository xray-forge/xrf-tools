import { useInjection } from "@wirestate/react";
import { IRendererShadowSettings } from "@xrf/renderer";
import { ReactElement } from "react";

import { SettingsService } from "@/core/settings/services/settings";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { ChoiceFormRow, IChoiceFormRowOption } from "@/core/ui/form/ChoiceFormRow";
import { SliderFormRow } from "@/core/ui/form/SliderFormRow";
import { DetailSection } from "@/core/ui/layout/DetailSection";
import { formatNumber } from "@/lib/format/number";

/** The engine's three cascades (`render_phase_sun.cpp`), and a fourth reaching three times as far. */
const CASCADE_WIDTHS: ReadonlyArray<number> = [20, 40, 160, 480];

const CASCADE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<string>> = CASCADE_WIDTHS.map((_, index: number) => ({
  label: String(index + 1),
  value: String(index + 1),
}));

const RESOLUTION_OPTIONS: ReadonlyArray<IChoiceFormRowOption<string>> = [1024, 2048, 4096].map((value: number) => ({
  label: String(value),
  value: String(value),
}));

/** The sun's shadow: how many cascades, how fine, and how they are filtered and drawn. */
export function SettingsRendererShadows(): ReactElement {
  const settingsService: SettingsService = useInjection(SettingsService);

  const shadows: IRendererShadowSettings = settingsService.rendererFeatures.shadows;

  function set(part: Partial<IRendererShadowSettings>): void {
    settingsService.setRendererOverrides({ shadows: part });
  }

  return (
    <DetailSection
      title={"Shadows"}
      description={
        "The sun's shadow in cascades of maps, the nearest the finest. The game's are three, 20, 40 and 160 metres " +
        "across at 2048 texels."
      }
    >
      <div className={"mt-4 flex flex-col gap-6"}>
        <CheckboxFormRow
          label={"Sun shadows"}
          description={"Off, the sun lights every surface facing it, as the level's baked light alone shades it."}
          isChecked={shadows.isEnabled}
          onChange={(isEnabled: boolean) => set({ isEnabled })}
        />

        <ChoiceFormRow
          label={"Cascades"}
          description={"How far the shadow reaches: 20, 40, 160, then 480 metres across."}
          options={CASCADE_OPTIONS}
          value={String(shadows.cascades.length)}
          onChange={(count: string) => set({ cascades: CASCADE_WIDTHS.slice(0, Number(count)) })}
        />

        <ChoiceFormRow
          label={"Map resolution"}
          description={"Texels each cascade's map is across. Finer edges cost more to draw and to hold."}
          options={RESOLUTION_OPTIONS}
          value={String(shadows.resolution)}
          onChange={(resolution: string) => set({ resolution: Number(resolution) })}
        />

        <SliderFormRow
          label={"Filter"}
          description={"Texels each way a shadow's edge is averaged over: none for the hardest edge."}
          value={shadows.filter}
          min={0}
          max={3}
          step={1}
          onChange={(filter: number) => set({ filter })}
        />

        <SliderFormRow
          label={"Normal offset"}
          description={
            "Texels a surface is moved off itself before it is compared, which keeps it from shadowing itself."
          }
          value={shadows.bias}
          min={0}
          max={5}
          step={0.25}
          format={(value: number) => formatNumber(value, 2)}
          onChange={(bias: number) => set({ bias })}
        />

        <SliderFormRow
          label={"Caster reach"}
          description={"Metres towards the sun past a cascade that its casters may stand, as tall as a tower is."}
          value={shadows.reach}
          min={100}
          max={1000}
          step={50}
          format={(value: number) => `${value} m`}
          onChange={(reach: number) => set({ reach })}
        />

        <CheckboxFormRow
          label={"Staggered redraws"}
          description={
            "Draws the farther cascades every second and fourth frame. Exact for everything standing still, and " +
            "what keeps a moving camera near the frame rate."
          }
          isChecked={shadows.isStaggered}
          onChange={(isStaggered: boolean) => set({ isStaggered })}
        />
      </div>
    </DetailSection>
  );
}
