/// <reference types="vite/client" />
import type { Meta, StoryObj } from "@storybook/react-vite";
import { ComponentType, ReactElement, useCallback, useState } from "react";
import { fn } from "storybook/test";

import { EditorPopoverAction } from "#/editor/popover-action";
import { EditorPopoverContent } from "#/editor/popover-content";
import { EditorPopoverSwitchRow } from "#/editor/popover-row";
import { EditorPopoverToggle } from "#/editor/popover-toggle";
import { GridOnIcon } from "#/icons/grid-on";
import { LayersIcon } from "#/icons/layers";
import { TuneIcon } from "#/icons/tune";
import { VisibilityIcon } from "#/icons/visibility";
import { FoundationSection } from "#/testing/stories";

type TIcon = ComponentType<{ className?: string }>;

/** Every pass-through module, each re-exporting one MUI icon as `<MuiName>Icon`. */
const ICON_MODULES: Record<string, Record<string, TIcon>> = import.meta.glob("./*.ts", { eager: true });

const ICONS: ReadonlyArray<[string, TIcon]> = Object.values(ICON_MODULES)
  .flatMap((module: Record<string, TIcon>) => Object.entries(module))
  .sort(([first], [second]) => first.localeCompare(second));

/** What the rows inside the cue's popovers report; they only show what a popover holds. */
const ON_ROW_CHANGE: (isChecked: boolean) => void = fn();

const ON_DISABLED_TOGGLE: () => void = fn();

interface IIconographyIconProps {
  name: string;
  Icon: TIcon;
}

function IconographyIcon({ name, Icon }: IIconographyIconProps): ReactElement {
  return (
    <div className={"flex items-center gap-2 rounded-control p-1.5 hover:bg-action-hover"}>
      <Icon className={"shrink-0 text-text-secondary"} />
      <code className={"truncate font-monospace text-xs"}>{name}</code>
    </div>
  );
}

/** Left to right: a popover trigger, a toggle on, a toggle off, and a toggle that cannot turn over. */
function IconographyCue(): ReactElement {
  const [isGridOn, setGridOn] = useState<boolean>(true);
  const [isLayerOn, setLayerOn] = useState<boolean>(false);

  const onGridToggle = useCallback(() => setGridOn((current: boolean) => !current), []);
  const onLayerToggle = useCallback(() => setLayerOn((current: boolean) => !current), []);

  return (
    <div className={"flex w-fit items-center gap-1 surface-frame p-1"}>
      <EditorPopoverAction label={"Display"} description={"Opens the display settings"} icon={<TuneIcon />}>
        <EditorPopoverContent label={"Display"}>
          <EditorPopoverSwitchRow label={"Show grid"} isChecked={isGridOn} onChange={setGridOn} />
        </EditorPopoverContent>
      </EditorPopoverAction>

      <EditorPopoverToggle
        label={"Grid"}
        description={isGridOn ? "Grid shown" : "Grid hidden"}
        icon={<GridOnIcon />}
        isOn={isGridOn}
        toggleLabel={"Show grid"}
        onToggle={onGridToggle}
      >
        <EditorPopoverSwitchRow label={"Minor lines"} isChecked={true} onChange={ON_ROW_CHANGE} />
      </EditorPopoverToggle>

      <EditorPopoverToggle
        label={"Layers"}
        description={isLayerOn ? "Layers shown" : "Layers hidden"}
        icon={<LayersIcon />}
        isOn={isLayerOn}
        toggleLabel={"Show layers"}
        onToggle={onLayerToggle}
      >
        <EditorPopoverSwitchRow label={"Labels"} isChecked={false} onChange={ON_ROW_CHANGE} />
      </EditorPopoverToggle>

      <EditorPopoverToggle
        label={"Visibility"}
        description={"Nothing loaded to show"}
        icon={<VisibilityIcon />}
        isOn={false}
        isDisabled
        toggleLabel={"Show objects"}
        onToggle={ON_DISABLED_TOGGLE}
      >
        {null}
      </EditorPopoverToggle>
    </div>
  );
}

const meta = {
  title: "Foundations/Iconography",
} satisfies Meta;

export default meta;

type TStory = StoryObj<typeof meta>;

export const Icons: TStory = {
  render: () => (
    <FoundationSection
      title={"Icons"}
      description={"The icons the kit passes through, named as the application imports them from `@xrf/ui/icons`."}
    >
      <div className={"grid grid-cols-[repeat(auto-fill,minmax(11rem,1fr))] gap-1"}>
        {ICONS.map(([name, Icon]) => (
          <IconographyIcon key={name} name={name} Icon={Icon} />
        ))}
      </div>
    </FoundationSection>
  ),
};

export const TriggerAndToggleCue: TStory = {
  render: () => (
    <FoundationSection
      title={"Trigger and toggle cue"}
      description={
        "Kind is shape, never hue: a corner triangle says a click opens something, a corner dot says it turns over, " +
        "and a toggle that cannot turn over draws the triangle again. The cue greys with a disabled control."
      }
    >
      <IconographyCue />
    </FoundationSection>
  ),
};
