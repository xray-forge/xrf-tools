import { IRenderFeatureSettings } from "@/core/render/lib/settings/render-feature-settings";

/** A group's override: part of the group, or the whole of a setting that is one value. */
type TRenderFeatureOverride<T> = T extends ReadonlyArray<unknown> ? T : T extends object ? Partial<T> : T;

/** Every feature's override, from the settings themselves, so the two cannot drift. */
type TRenderFeatureOverrideGroups = {
  [K in keyof IRenderFeatureSettings]?: TRenderFeatureOverride<IRenderFeatureSettings[K]>;
};

/** What was changed on top of a preset, feature by feature. */
export interface IRenderFeatureOverrides extends TRenderFeatureOverrideGroups {}
