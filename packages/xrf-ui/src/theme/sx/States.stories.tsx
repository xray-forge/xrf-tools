import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";

import { IconButton } from "#/button";
import { GridOnIcon } from "#/icons/grid-on";
import { VisibilityIcon } from "#/icons/visibility";
import { FoundationSection } from "#/testing/stories";
import { CONTROL, STATE } from "#/theme";

import { getControlStateSx } from "./control-state";

/** The states drawn as a scrim over a level; the rest of `STATE` are MUI's own opacities. */
type TStateLadderName = "hover" | "current" | "selected" | "well";

interface IStateLadderRung {
  name: TStateLadderName;
  /** The utility the state paints through. */
  className: string;
  note: string;
}

interface IStateControlCase {
  label: string;
  isActive: boolean;
  isDisabled: boolean;
}

const LADDER: ReadonlyArray<IStateLadderRung> = [
  { name: "hover", className: "bg-action-hover", note: "Under the pointer." },
  { name: "current", className: "bg-action-current", note: "What the editor has open, or chrome that is playing." },
  { name: "selected", className: "bg-action-selected", note: "Where the keyboard stands; the top of the ladder." },
  { name: "well", className: "bg-well border border-divider", note: "A recess, toward black in both schemes." },
];

const LADDER_SURFACES: ReadonlyArray<string> = ["surface-content", "surface-frame"];

const CONTROL_CASES: ReadonlyArray<IStateControlCase> = [
  { label: "rest", isActive: false, isDisabled: false },
  { label: "active", isActive: true, isDisabled: false },
  { label: "disabled", isActive: false, isDisabled: true },
  { label: "disabled, active", isActive: true, isDisabled: true },
];

const CONTROL_ICONS = [
  { name: "visibility", Icon: VisibilityIcon },
  { name: "grid", Icon: GridOnIcon },
] as const;

function toShare(value: (typeof STATE)[TStateLadderName]): string {
  return `dark ${value.dark} / light ${value.light}`;
}

interface IStateLadderProps {
  surface: string;
}

function StateLadder({ surface }: IStateLadderProps): ReactElement {
  return (
    <div className={`flex flex-col gap-2 rounded-surface border border-divider p-3 ${surface}`}>
      <code className={"font-monospace text-xs text-text-secondary"}>{surface}</code>

      {LADDER.map(({ name, className }: IStateLadderRung) => (
        <div key={name} className={`flex h-7 items-center rounded-control px-2 text-sm ${className}`}>
          {name}
        </div>
      ))}
    </div>
  );
}

function StateControl({ label, isActive, isDisabled }: IStateControlCase): ReactElement {
  return (
    <div className={"flex flex-col items-center gap-1"}>
      <div className={"flex gap-1"}>
        {CONTROL_ICONS.map(({ name, Icon }) => (
          <IconButton
            key={name}
            aria-label={`${label} ${name}`}
            className={"h-editor-action w-editor-action p-0"}
            disabled={isDisabled}
            sx={{ ...getControlStateSx(isActive), "& svg": { fontSize: CONTROL.editorActionIconSize } }}
          >
            <Icon />
          </IconButton>
        ))}
      </div>
      <span className={"text-xs text-text-secondary"}>{label}</span>
    </div>
  );
}

const meta = {
  title: "Foundations/States",
} satisfies Meta;

export default meta;

type TStory = StoryObj<typeof meta>;

export const Ladder: TStory = {
  render: () => (
    <FoundationSection
      title={"Ladder"}
      description={"States are translucent scrims over a level, never fills, so each keeps the wash of either level."}
    >
      <div className={"grid grid-cols-2 gap-4"}>
        {LADDER_SURFACES.map((surface: string) => (
          <StateLadder key={surface} surface={surface} />
        ))}
      </div>

      <dl className={"grid grid-cols-[8rem_14rem_1fr] gap-x-4 gap-y-1 text-xs"}>
        {LADDER.map(({ name, note }: IStateLadderRung) => (
          <div key={name} className={"contents"}>
            <dt className={"font-monospace"}>{`STATE.${name}`}</dt>
            <dd className={"font-monospace text-text-secondary"}>{toShare(STATE[name])}</dd>
            <dd className={"text-text-secondary"}>{note}</dd>
          </div>
        ))}
      </dl>
    </FoundationSection>
  ),
};

/** Hover and tab through the controls: hover and the focus ring are the same feedback in every state. */
export const ControlPaint: TStory = {
  render: () => (
    <FoundationSection
      title={"Control paint"}
      description={"`getControlStateSx`: rest, active and disabled are colours rather than scrims or fades."}
    >
      <div className={"flex gap-8 surface-frame p-4"}>
        {CONTROL_CASES.map((control: IStateControlCase) => (
          <StateControl key={control.label} {...control} />
        ))}
      </div>
    </FoundationSection>
  ),
};
