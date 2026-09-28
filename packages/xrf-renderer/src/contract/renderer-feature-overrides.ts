import { IRendererFeatureSettings } from "#/contract/renderer-feature-settings";

/** A group's override: part of the group, or the whole of a setting that is one value. */
type TRendererFeatureOverride<T> = T extends ReadonlyArray<unknown> ? T : T extends object ? Partial<T> : T;

/** Every feature's override, from the settings themselves, so the two cannot drift. */
type TRendererFeatureOverrideGroups = {
  [K in keyof IRendererFeatureSettings]?: TRendererFeatureOverride<IRendererFeatureSettings[K]>;
};

/** What was changed on top of a preset, feature by feature. */
export interface IRendererFeatureOverrides extends TRendererFeatureOverrideGroups {}
