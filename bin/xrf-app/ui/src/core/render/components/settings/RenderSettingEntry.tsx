import { BaseComponentProps } from "@xrf/ui/element-types";
import { CheckboxFormRow } from "@xrf/ui/form/checkbox-row";
import { IChoiceFormRowOption } from "@xrf/ui/form/choice-row";
import { ReactElement, useCallback } from "react";

import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  describeRenderSetting,
  ERenderSettingControl,
  isRenderSettingInert,
  TRenderSettings,
} from "@/core/render/lib/features/render-setting";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/choice/render-feature-settings";

export interface IRenderSettingEntryProps<G, K extends keyof G> extends BaseComponentProps {
  /** How each setting of the group is set. */
  settings: TRenderSettings<G>;
  /** The setting the entry sets. */
  name: K;
  /** The group as the view draws it. */
  group: G;
  /** Every feature as the view draws them, which a setting's rule reads. */
  features: IRenderFeatureSettings;
  /** Whether it shows disabled whatever its rule says, where something outside the features turns it off. */
  isDisabled?: boolean;
  /** The values a choice offers, where something outside the features narrows the declared ones, as dev mode does. */
  options?: ReadonlyArray<IChoiceFormRowOption<string>>;
  /** Sets part of the group, the rest as it was. */
  onSet: (part: Partial<G>) => void;
}

/**
 * One setting's entry in a popover, as its declaration says: compact, its label alone unless the declaration has it
 * say what it does, and disabled where it changes nothing. A setting drawn by a control of its own draws nothing here.
 */
export function RenderSettingEntry<G, K extends keyof G>({
  "data-testid": dataTestId = "render-setting-entry",
  id,
  className,
  settings,
  name,
  group,
  features,
  isDisabled = false,
  options,
  onSet,
}: IRenderSettingEntryProps<G, K>): ReactElement {
  const setting = settings[name];
  const value: G[K] = group[name];
  const label: string = setting.entryLabel ?? setting.label;
  const description: string | undefined = setting.isEntryDescribed
    ? describeRenderSetting(setting, group, features)
    : undefined;
  const isOff: boolean = isDisabled || isRenderSettingInert(setting, group, features);
  const onChange = useCallback((next: G[K]): void => onSet({ [name]: next } as unknown as Partial<G>), [name, onSet]);

  switch (setting.control) {
    case ERenderSettingControl.CHECKBOX:
      return (
        <CheckboxFormRow
          data-testid={dataTestId}
          id={id}
          className={className}
          label={label}
          description={description}
          isChecked={value as boolean}
          isDisabled={isOff}
          onChange={(isChecked: boolean) => onChange(isChecked as G[K])}
        />
      );

    case ERenderSettingControl.CHOICE:
      return (
        <>
          <RenderValueChoice
            data-testid={dataTestId}
            id={id}
            className={className}
            label={label}
            options={options ?? setting.options}
            value={value as string}
            isStacked={setting.isStacked}
            isDisabled={isOff}
            onChange={(chosen: string) => onChange(chosen as G[K])}
          />

          {setting.isEntryDescribed ? (
            <p className={"text-xs text-text-secondary"}>{setting.explain?.(value as never) ?? description}</p>
          ) : null}
        </>
      );

    case ERenderSettingControl.SLIDER:
      return (
        <>
          <RenderValueSlider
            data-testid={dataTestId}
            id={id}
            className={className}
            label={label}
            value={value as number}
            {...setting.limits}
            isDisabled={isOff}
            format={setting.format}
            onChange={(moved: number) => onChange(moved as G[K])}
          />

          {description ? <p className={"text-xs text-text-secondary"}>{description}</p> : null}
        </>
      );

    case ERenderSettingControl.CUSTOM:
      return <></>;
  }
}
