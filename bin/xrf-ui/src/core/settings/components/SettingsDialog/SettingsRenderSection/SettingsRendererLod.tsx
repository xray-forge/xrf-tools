import { useInjection } from "@wirestate/react";
import { IRendererLodSettings } from "@xrf/renderer";
import { ReactElement } from "react";

import { SettingsService } from "@/core/settings/services/settings";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { SliderFormRow } from "@/core/ui/form/SliderFormRow";
import { DetailSection } from "@/core/ui/layout/DetailSection";
import { formatNumber } from "@/lib/format/number";

/** One threshold, the console variable it is and the range the engine's console takes it in. */
interface ILodThreshold {
  key: Exclude<keyof IRendererLodSettings, "isImpostors">;
  label: string;
  description: string;
  min: number;
  max: number;
  step: number;
  digits: number;
}

/** In the order the engine's console lists them (`xrRender_console.cpp`), with its own ranges. */
const LOD_THRESHOLDS: ReadonlyArray<ILodThreshold> = [
  {
    description: "r__geometry_lod: every threshold below is taken against the drawing's area times this.",
    digits: 2,
    key: "geometryLod",
    label: "Detail scale",
    max: 2,
    min: 0.1,
    step: 0.05,
  },
  {
    description: "r2_ssa_lod_a: a clump smaller on screen than this draws as its impostor.",
    digits: 0,
    key: "ssaA",
    label: "Impostor below",
    max: 96,
    min: 16,
    step: 1,
  },
  {
    description: "r2_ssa_lod_b: a clump larger than this draws its trees; between the two, both.",
    digits: 0,
    key: "ssaB",
    label: "Trees above",
    max: 64,
    min: 32,
    step: 1,
  },
  {
    description: "r__ssa_discard: a clump smaller than this draws nothing at all.",
    digits: 1,
    key: "ssaDiscard",
    label: "Nothing below",
    max: 10,
    min: 1,
    step: 0.5,
  },
  {
    description: "r__ssa_glod_start: a progressive tree larger than this draws its whole detail.",
    digits: 0,
    key: "ssaGlodStart",
    label: "Whole detail above",
    max: 512,
    min: 128,
    step: 8,
  },
  {
    description: "r__ssa_glod_end: a progressive tree smaller than this draws its coarsest band.",
    digits: 0,
    key: "ssaGlodEnd",
    label: "Coarsest below",
    max: 96,
    min: 16,
    step: 1,
  },
];

/** When a clump of trees draws as its impostor, and a progressive tree at less than its whole detail. */
export function SettingsRendererLod(): ReactElement {
  const settingsService: SettingsService = useInjection(SettingsService);

  const lod: IRendererLodSettings = settingsService.rendererFeatures.lod;

  return (
    <DetailSection
      title={"Levels of detail"}
      description={
        "The engine's own thresholds on how much of the screen a tree covers. A level's toolbar can move its " +
        "impostors nearer or further on top of these, for that view alone."
      }
    >
      <div className={"mt-4 flex flex-col gap-6"}>
        <CheckboxFormRow
          label={"Impostors"}
          description={"Draws a distant clump of trees as its impostor. Off, every tree draws in full at any distance."}
          isChecked={lod.isImpostors}
          onChange={(isImpostors: boolean) => settingsService.setRendererOverrides({ lod: { isImpostors } })}
        />

        {LOD_THRESHOLDS.map((threshold: ILodThreshold) => (
          <SliderFormRow
            key={threshold.key}
            label={threshold.label}
            description={threshold.description}
            value={lod[threshold.key]}
            min={threshold.min}
            max={threshold.max}
            step={threshold.step}
            format={(value: number) => formatNumber(value, threshold.digits)}
            onChange={(value: number) => settingsService.setRendererOverrides({ lod: { [threshold.key]: value } })}
          />
        ))}
      </div>
    </DetailSection>
  );
}
