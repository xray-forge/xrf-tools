import { EditorPopoverGroupSection } from "@xrf/ui/editor/popover-group";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement, ReactNode } from "react";

import { RenderSettingEntries } from "@/core/render/components/settings/RenderSettingEntries";
import {
  describeRenderSetting,
  isRenderSettingInert,
  TRenderSettings,
} from "@/core/render/lib/features/render-setting";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/choice/render-feature-settings";

/** A group's settings that are on or off, any of which can head a section of the values it turns on. */
export type TRenderSwitchKey<G> = { [K in keyof G]: G[K] extends boolean ? K : never }[keyof G];

export interface IRenderSettingSectionProps<G> extends BaseComponentProps {
  /** How each setting of the group is set. */
  settings: TRenderSettings<G>;
  /** The switch heading the section, its box before the section's name. */
  name: TRenderSwitchKey<G>;
  /** The values it turns on, as their entries, top to bottom. */
  names: ReadonlyArray<keyof G>;
  /** The group as the view draws it. */
  group: G;
  /** Every feature as the view draws them, which a setting's rule reads. */
  features: IRenderFeatureSettings;
  /** What the mode the section's values stand at draws, said under its heading. */
  note?: string;
  /** Sets part of the group, the rest as it was. */
  onSet: (part: Partial<G>) => void;
  /** What acts on the section beyond its values, such as a pick in the view, at the end of its heading. */
  actions?: ReactNode;
}

/**
 * One switch of a group heading a popover section of the values it turns on, as its declaration says: the switch's
 * name, its description read on hovering it, disabled where its rule finds it changing nothing; the values follow as
 * their entries, each disabled while the switch is off by its own rule.
 */
export function RenderSettingSection<G>({
  "data-testid": dataTestId = "render-setting-section",
  id,
  className,
  settings,
  name,
  names,
  group,
  features,
  note,
  onSet,
  actions,
}: IRenderSettingSectionProps<G>): ReactElement {
  const setting = settings[name];
  const isOn: boolean = group[name] as boolean;

  return (
    <EditorPopoverGroupSection
      data-testid={dataTestId}
      id={id}
      className={className}
      label={setting.entryLabel ?? setting.label}
      description={describeRenderSetting(setting, group, features)}
      note={note}
      isOn={isOn}
      isDisabled={isRenderSettingInert(setting, group, features)}
      actions={actions}
      onToggle={() => onSet({ [name]: !isOn } as unknown as Partial<G>)}
    >
      <RenderSettingEntries settings={settings} names={names} group={group} features={features} onSet={onSet} />
    </EditorPopoverGroupSection>
  );
}
