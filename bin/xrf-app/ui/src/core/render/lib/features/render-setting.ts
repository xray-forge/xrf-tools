import { IChoiceFormRowOption } from "@xrf/ui/form/choice-row";

import { IRenderLimits } from "@/core/render/lib/features/render-limits";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/choice/render-feature-settings";

/** How a setting is set: what its row in the settings and its entry in a popover draw. */
export enum ERenderSettingControl {
  CHECKBOX = "checkbox",
  CHOICE = "choice",
  SLIDER = "slider",
  /** A control of its own, which reads only the setting's words and rule. */
  CUSTOM = "custom",
}

/** What every setting says wherever it is set, and when it changes nothing. */
interface IRenderSettingText<G> {
  /** What it is called, in the settings and in a popover alike. */
  label: string;
  /**
   * What it is called in a popover, where that differs: shorter under a section heading naming what it leaves out, or
   * its feature named where the popover holds more than its own.
   */
  entryLabel?: string;
  /** What it does, under its row in the settings; a function where that turns on its group or another feature. */
  description?: string | ((group: G, features: IRenderFeatureSettings) => string);
  /** Whether a popover's entry says what it does too, as the settings' row always does. */
  isEntryDescribed?: boolean;
  /** Whether it changes nothing with its group and every feature as drawn, where it shows disabled. */
  isInert?: (group: G, features: IRenderFeatureSettings) => boolean;
}

/** A setting on or off. */
export interface IRenderCheckboxSetting<G> extends IRenderSettingText<G> {
  control: ERenderSettingControl.CHECKBOX;
}

/** One of a few values. */
export interface IRenderChoiceSetting<G, T extends string> extends IRenderSettingText<G> {
  control: ERenderSettingControl.CHOICE;
  options: ReadonlyArray<IChoiceFormRowOption<T>>;
  /** What the value chosen does, said in place of a fixed description. */
  explain?: (value: T) => string;
  /** Whether a popover stacks its values, for names too long to share a row. */
  isStacked?: boolean;
}

/** A number between bounds. */
export interface IRenderSliderSetting<G> extends IRenderSettingText<G> {
  control: ERenderSettingControl.SLIDER;
  limits: IRenderLimits;
  /** How the value reads, the only place its unit is stated. */
  format: (value: number) => string;
}

/** A setting drawn by a control of its own, such as a count of cascades standing for their widths. */
export interface IRenderCustomSetting<G> extends IRenderSettingText<G> {
  control: ERenderSettingControl.CUSTOM;
}

/** How one setting of a group is set, by the kind of value it holds. */
export type TRenderSetting<G, V> = [V] extends [boolean]
  ? IRenderCheckboxSetting<G> | IRenderCustomSetting<G>
  : [V] extends [number]
    ? IRenderSliderSetting<G> | IRenderCustomSetting<G>
    : [V] extends [string]
      ? IRenderChoiceSetting<G, V> | IRenderCustomSetting<G>
      : IRenderCustomSetting<G>;

/** How each setting of a group is set: one declaration the settings' rows and a popover's entries both draw. */
export type TRenderSettings<G> = { readonly [K in keyof G]: TRenderSetting<G, G[K]> };

/**
 * @param setting - How a setting is set.
 * @param group - Its group as drawn.
 * @param features - Every feature as drawn.
 * @returns What it does as they are, or none where it says nothing.
 */
export function describeRenderSetting<G>(
  setting: IRenderSettingText<G>,
  group: G,
  features: IRenderFeatureSettings
): string | undefined {
  return typeof setting.description === "function" ? setting.description(group, features) : setting.description;
}

/**
 * @param setting - How a setting is set.
 * @param group - Its group as drawn.
 * @param features - Every feature as drawn.
 * @returns Whether it changes nothing as they are, where it shows disabled.
 */
export function isRenderSettingInert<G>(
  setting: IRenderSettingText<G>,
  group: G,
  features: IRenderFeatureSettings
): boolean {
  return setting.isInert?.(group, features) ?? false;
}
