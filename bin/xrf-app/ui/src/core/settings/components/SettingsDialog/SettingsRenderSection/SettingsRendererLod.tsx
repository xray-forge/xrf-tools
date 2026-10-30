import { useInjection } from "@wirestate/react";
import { formatNumber } from "@xrf/format";
import { ERenderImpostorMode, RenderImpostorMode } from "@xrf/ipc/types/xrf-renderer";
import { CheckboxFormRow } from "@xrf/ui/form/checkbox-row";
import { ChoiceFormRow } from "@xrf/ui/form/choice-row";
import { SliderFormRow } from "@xrf/ui/form/slider-row";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { ReactElement } from "react";

import {
  explainRenderImpostorMode,
  explainRenderImpostorQuality,
  formatImpostorShare,
  RENDER_IMPOSTOR_MODE_OPTIONS,
  RENDER_IMPOSTOR_QUALITY_OPTIONS,
  RENDER_IMPOSTOR_SHARE_LIMITS,
  toRenderImpostorQuality,
  TRenderImpostorQuality,
} from "@/core/render/lib/features/render-impostor-choices";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/render-feature-schema";
import { TRenderLodSettings } from "@/core/render/lib/settings/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

/** One threshold, and the console variable it is. */
interface ILodThreshold {
  key: "geometryLod" | "ssaA" | "ssaB" | "ssaDiscard" | "ssaGlodStart" | "ssaGlodEnd";
  label: string;
  description: string;
  step: number;
  digits: number;
}

/** In the order the engine's console lists them (`xrRender_console.cpp`); its ranges are the renderer's schema's. */
const LOD_THRESHOLDS: ReadonlyArray<ILodThreshold> = [
  {
    description: "Scales every threshold below. Game setting: r__geometry_lod.",
    digits: 2,
    key: "geometryLod",
    label: "Detail scale",
    step: 0.05,
  },
  {
    description: "Clusters smaller on screen draw as impostors. Game setting: r2_ssa_lod_a.",
    digits: 0,
    key: "ssaA",
    label: "Impostor below",
    step: 1,
  },
  {
    description: "Clusters larger draw as trees; in between, both. Game setting: r2_ssa_lod_b.",
    digits: 0,
    key: "ssaB",
    label: "Trees above",
    step: 1,
  },
  {
    description: "Clusters smaller are not drawn. Game setting: r__ssa_discard.",
    digits: 1,
    key: "ssaDiscard",
    label: "Cull below",
    step: 0.5,
  },
  {
    description: "Progressive trees larger draw at full detail. Game setting: r__ssa_glod_start.",
    digits: 0,
    key: "ssaGlodStart",
    label: "Full detail above",
    step: 8,
  },
  {
    description: "Progressive trees smaller draw at the lowest detail. Game setting: r__ssa_glod_end.",
    digits: 0,
    key: "ssaGlodEnd",
    label: "Lowest detail below",
    step: 1,
  },
];

/**
 * When a tree draws as its impostor and whose impostors they are, the engine's or baked from each tree type with their
 * quality and how many trees get it; and a progressive tree at less than its whole detail.
 */
export function SettingsRendererLod(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const lod: TRenderLodSettings = renderSettingsService.rendererFeatures.lod;
  const isBaked: boolean = lod.impostorMode === ERenderImpostorMode.BAKED;
  const quality: TRenderImpostorQuality = toRenderImpostorQuality(lod.impostorSize);

  return (
    <DetailSection
      title={"Levels of detail"}
      description={"Engine thresholds by screen coverage. The level toolbar can offset impostor distance per view."}
    >
      <div className={"mt-4 flex flex-col gap-6"}>
        <CheckboxFormRow
          label={"Impostors"}
          description={"Draws distant trees as impostors. Off, every tree renders at full detail."}
          isChecked={lod.isImpostors}
          onChange={(isImpostors: boolean) => renderSettingsService.setRendererOverrides({ lod: { isImpostors } })}
        />

        <ChoiceFormRow
          label={"Impostor kind"}
          description={explainRenderImpostorMode(lod.impostorMode)}
          options={RENDER_IMPOSTOR_MODE_OPTIONS}
          value={lod.impostorMode}
          isDisabled={!lod.isImpostors}
          onChange={(impostorMode: RenderImpostorMode) =>
            renderSettingsService.setRendererOverrides({ lod: { impostorMode } })
          }
        />

        <ChoiceFormRow
          label={"Baked quality"}
          description={explainRenderImpostorQuality(quality)}
          options={RENDER_IMPOSTOR_QUALITY_OPTIONS}
          value={quality}
          isDisabled={!isBaked}
          onChange={(value: TRenderImpostorQuality) =>
            renderSettingsService.setRendererOverrides({ lod: { impostorSize: Number(value) } })
          }
        />

        <SliderFormRow
          label={"Full quality for"}
          description={
            "The most common tree types bake at the full quality until they cover this share of the level's " +
            "trees; the rest bake at half."
          }
          value={lod.impostorShare}
          {...RENDER_IMPOSTOR_SHARE_LIMITS}
          isDisabled={!isBaked}
          format={formatImpostorShare}
          onChange={(impostorShare: number) => renderSettingsService.setRendererOverrides({ lod: { impostorShare } })}
        />

        {LOD_THRESHOLDS.map((threshold: ILodThreshold) => (
          <SliderFormRow
            key={threshold.key}
            label={threshold.label}
            description={threshold.description}
            value={lod[threshold.key]}
            min={RENDER_FEATURE_SCHEMA.lod[threshold.key].min}
            max={RENDER_FEATURE_SCHEMA.lod[threshold.key].max}
            step={threshold.step}
            format={(value: number) => formatNumber(value, threshold.digits)}
            onChange={(value: number) =>
              renderSettingsService.setRendererOverrides({ lod: { [threshold.key]: value } })
            }
          />
        ))}
      </div>
    </DetailSection>
  );
}
