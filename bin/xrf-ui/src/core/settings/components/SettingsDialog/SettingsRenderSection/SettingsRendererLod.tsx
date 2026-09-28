import { useInjection } from "@wirestate/react";
import { IRendererLodSettings, RENDERER_FEATURE_SCHEMA } from "@xrf/renderer";
import { ReactElement } from "react";

import { SettingsService } from "@/core/settings/services/settings";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { SliderFormRow } from "@/core/ui/form/SliderFormRow";
import { DetailSection } from "@/core/ui/layout/DetailSection";
import { formatNumber } from "@/lib/format/number";

/** One threshold, and the console variable it is. */
interface ILodThreshold {
  key: Exclude<keyof IRendererLodSettings, "isImpostors">;
  label: string;
  description: string;
  step: number;
  digits: number;
}

/** In the order the engine's console lists them (`xrRender_console.cpp`); its ranges are the renderer's schema's. */
const LOD_THRESHOLDS: ReadonlyArray<ILodThreshold> = [
  {
    description: "Scales every threshold below (r__geometry_lod).",
    digits: 2,
    key: "geometryLod",
    label: "Detail scale",
    step: 0.05,
  },
  {
    description: "Clusters smaller on screen draw as impostors (r2_ssa_lod_a).",
    digits: 0,
    key: "ssaA",
    label: "Impostor below",
    step: 1,
  },
  {
    description: "Clusters larger draw as trees; in between, both (r2_ssa_lod_b).",
    digits: 0,
    key: "ssaB",
    label: "Trees above",
    step: 1,
  },
  {
    description: "Clusters smaller are not drawn (r__ssa_discard).",
    digits: 1,
    key: "ssaDiscard",
    label: "Cull below",
    step: 0.5,
  },
  {
    description: "Progressive trees larger draw at full detail (r__ssa_glod_start).",
    digits: 0,
    key: "ssaGlodStart",
    label: "Full detail above",
    step: 8,
  },
  {
    description: "Progressive trees smaller draw at the lowest detail (r__ssa_glod_end).",
    digits: 0,
    key: "ssaGlodEnd",
    label: "Lowest detail below",
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
      description={"Engine thresholds by screen coverage. The level toolbar can offset impostor distance per view."}
    >
      <div className={"mt-4 flex flex-col gap-6"}>
        <CheckboxFormRow
          label={"Impostors"}
          description={"Draws distant tree clusters as impostors. Off, every tree renders at full detail."}
          isChecked={lod.isImpostors}
          onChange={(isImpostors: boolean) => settingsService.setRendererOverrides({ lod: { isImpostors } })}
        />

        {LOD_THRESHOLDS.map((threshold: ILodThreshold) => (
          <SliderFormRow
            key={threshold.key}
            label={threshold.label}
            description={threshold.description}
            value={lod[threshold.key]}
            min={RENDERER_FEATURE_SCHEMA.lod[threshold.key].min}
            max={RENDERER_FEATURE_SCHEMA.lod[threshold.key].max}
            step={threshold.step}
            format={(value: number) => formatNumber(value, threshold.digits)}
            onChange={(value: number) => settingsService.setRendererOverrides({ lod: { [threshold.key]: value } })}
          />
        ))}
      </div>
    </DetailSection>
  );
}
