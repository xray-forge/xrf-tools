import { BaseComponentProps } from "@xrf/ui/element-types";
import { CheckboxFormRow } from "@xrf/ui/form/checkbox-row";
import { ChoiceFormRow, IChoiceFormRowOption } from "@xrf/ui/form/choice-row";
import { SliderFormRow } from "@xrf/ui/form/slider-row";
import { ReactElement, useCallback } from "react";

import {
  describeRenderSetting,
  ERenderSettingControl,
  isRenderSettingInert,
  TRenderSettings,
} from "@/core/render/lib/features/render-setting";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/choice/render-feature-settings";

export interface IRenderSettingRowProps<G, K extends keyof G> extends BaseComponentProps {
  /** How each setting of the group is set. */
  settings: TRenderSettings<G>;
  /** The setting the row sets. */
  name: K;
  /** The group as the settings resolve it. */
  group: G;
  /** Every feature as the settings resolve them, which a setting's rule reads. */
  features: IRenderFeatureSettings;
  /** Whether it shows disabled whatever its rule says, where something outside the features turns it off. */
  isDisabled?: boolean;
  /** The values a choice offers, where something outside the features narrows the declared ones, as dev mode does. */
  options?: ReadonlyArray<IChoiceFormRowOption<string>>;
  /** Sets part of the group, the rest as it was. */
  onSet: (part: Partial<G>) => void;
}

/**
 * One setting's row in the settings, as its declaration says: its label, what it does under it, and disabled where it
 * changes nothing. A setting drawn by a control of its own draws nothing here.
 */
export function RenderSettingRow<G, K extends keyof G>({
  "data-testid": dataTestId = "render-setting-row",
  id,
  className,
  settings,
  name,
  group,
  features,
  isDisabled = false,
  options,
  onSet,
}: IRenderSettingRowProps<G, K>): ReactElement {
  const setting = settings[name];
  const value: G[K] = group[name];
  const description: string | undefined = describeRenderSetting(setting, group, features);
  const isOff: boolean = isDisabled || isRenderSettingInert(setting, group, features);
  const onChange = useCallback((next: G[K]): void => onSet({ [name]: next } as unknown as Partial<G>), [name, onSet]);

  switch (setting.control) {
    case ERenderSettingControl.CHECKBOX:
      return (
        <CheckboxFormRow
          data-testid={dataTestId}
          id={id}
          className={className}
          label={setting.label}
          description={description}
          isChecked={value as boolean}
          isDisabled={isOff}
          onChange={(isChecked: boolean) => onChange(isChecked as G[K])}
        />
      );

    case ERenderSettingControl.CHOICE:
      return (
        <ChoiceFormRow
          data-testid={dataTestId}
          id={id}
          className={className}
          label={setting.label}
          description={setting.explain?.(value as never) ?? description}
          options={options ?? setting.options}
          value={value as string}
          isDisabled={isOff}
          onChange={(chosen: string) => onChange(chosen as G[K])}
        />
      );

    case ERenderSettingControl.SLIDER:
      return (
        <SliderFormRow
          data-testid={dataTestId}
          id={id}
          className={className}
          label={setting.label}
          description={description}
          value={value as number}
          {...setting.limits}
          isDisabled={isOff}
          format={setting.format}
          onChange={(moved: number) => onChange(moved as G[K])}
        />
      );

    case ERenderSettingControl.CUSTOM:
      return <></>;
  }
}
