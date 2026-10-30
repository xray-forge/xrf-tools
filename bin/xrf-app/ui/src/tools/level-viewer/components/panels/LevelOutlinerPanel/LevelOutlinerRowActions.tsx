import { LevelObjectKind, LevelObjectKindEntry } from "@xrf/ipc/types/xrf-app-levels";
import { Nullable } from "@xrf/types";
import { EditorViewToggle } from "@xrf/ui/editor/view-toggle";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ViewInArIcon } from "@xrf/ui/icons/view-in-ar";
import { VisibilityIcon } from "@xrf/ui/icons/visibility";
import { VisibilityOffIcon } from "@xrf/ui/icons/visibility-off";
import { cn } from "@xrf/ui/theme";
import { ReactElement } from "react";

import { getLevelVisibleOption, TLevelVisibleOption } from "@/core/level/lib/objects/level-object-visibility";

interface ILevelOutlinerKindActionsProps extends BaseComponentProps {
  /** What the row stands for, by name, as the tooltips say it. */
  label: string;
  /** The kinds the row switches: one for a kind's row, a family's for a group's. */
  kinds: ReadonlyArray<LevelObjectKindEntry>;
  /** Whether a kind is drawn, by the view option its Visible switch is. */
  isVisible: (option: TLevelVisibleOption) => boolean;
  /** Whether a kind's helper is switched on. */
  isHelped: (kind: LevelObjectKind) => boolean;
  /** Whether helpers are drawn at all, which the helper switches wait on. */
  isHelpersShown: boolean;
  onVisibleChange: (options: ReadonlyArray<TLevelVisibleOption>, isOn: boolean) => void;
  onHelperChange: (kinds: ReadonlyArray<LevelObjectKind>, isOn: boolean) => void;
}

/** How many of a row's kinds a switch has on: all, some, or none. */
function toShare(flags: ReadonlyArray<boolean>): "all" | "some" | "none" {
  const on: number = flags.filter(Boolean).length;

  return on === flags.length ? "all" : on ? "some" : "none";
}

/**
 * The Visible and Helper switches on a kind's or a group's row: a group's lit where all its kinds are, half lit where
 * some are, and a click turning every kind on unless all are already.
 */
export function LevelOutlinerKindActions({
  "data-testid": dataTestId = "level-outliner-kind-actions",
  id,
  className,
  label,
  kinds,
  isVisible,
  isHelped,
  isHelpersShown,
  onVisibleChange,
  onHelperChange,
}: ILevelOutlinerKindActionsProps): ReactElement {
  const options: Array<TLevelVisibleOption> = kinds
    .map((entry: LevelObjectKindEntry) => getLevelVisibleOption(entry.kind))
    .filter((option: Nullable<TLevelVisibleOption>): option is TLevelVisibleOption => option !== null);
  const visible = toShare(options.map(isVisible));
  const helped = toShare(kinds.map((entry: LevelObjectKindEntry) => isHelped(entry.kind)));
  const name: string = label.toLowerCase();

  return (
    <div data-testid={dataTestId} id={id} className={cn("flex items-center", className)}>
      <EditorViewToggle
        data-testid={"level-outliner-helper-toggle"}
        className={cn(helped === "some" && "opacity-60")}
        label={`Helpers of ${name}`}
        description={
          helped === "none" ? `No ${name} helpers` : `${name} helpers ${helped === "some" ? "partly " : ""}drawn`
        }
        icon={<ViewInArIcon />}
        isOn={helped !== "none"}
        isDisabled={!isHelpersShown}
        unavailableTitle={"Helpers are off: turn them on with G"}
        onToggle={() =>
          onHelperChange(
            kinds.map((entry: LevelObjectKindEntry) => entry.kind),
            helped !== "all"
          )
        }
      />

      {options.length ? (
        <EditorViewToggle
          data-testid={"level-outliner-visible-toggle"}
          className={cn(visible === "some" && "opacity-60")}
          label={`Show ${name}`}
          description={visible === "none" ? `${label} hidden` : `${label} ${visible === "some" ? "partly " : ""}drawn`}
          icon={visible === "none" ? <VisibilityOffIcon /> : <VisibilityIcon />}
          isOn={visible !== "none"}
          onToggle={() => onVisibleChange(options, visible !== "all")}
        />
      ) : null}
    </div>
  );
}

interface ILevelOutlinerObjectActionsProps extends BaseComponentProps {
  name: string;
  isHidden: boolean;
  onToggle: () => void;
}

/** The eye on one object's row, which hides it until revealed, as `H` hides the selection. */
export function LevelOutlinerObjectActions({
  "data-testid": dataTestId = "level-outliner-object-actions",
  id,
  className,
  name,
  isHidden,
  onToggle,
}: ILevelOutlinerObjectActionsProps): ReactElement {
  return (
    <EditorViewToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={`Hide ${name}`}
      description={isHidden ? `${name} hidden` : `${name} shown`}
      icon={isHidden ? <VisibilityOffIcon /> : <VisibilityIcon />}
      isOn={!isHidden}
      onToggle={onToggle}
    />
  );
}
