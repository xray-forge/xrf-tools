import { RENDER_FEATURE_SCHEMA as RENDER_CONTRACT_FEATURE_SCHEMA } from "@xrf/ipc/types/xrf-renderer";

import { IRenderFeatureSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { TRenderSettingSchema } from "@/core/render/lib/settings/schema/render-setting-schema";

/**
 * Every feature setting, what it takes, and between which bounds, as the renderer's contract describes each beside its
 * field (`RenderSettingSchema`), keyed as the feature settings are.
 */
export const RENDER_FEATURE_SCHEMA: TRenderSettingSchema<IRenderFeatureSettings> =
  RENDER_CONTRACT_FEATURE_SCHEMA as unknown as TRenderSettingSchema<IRenderFeatureSettings>;
