import { useInjection } from "@wirestate/react";
import { LEVEL_OBJECT_KINDS, LevelObjectKindEntry } from "@xrf/ipc/types/xrf-app-levels";
import { Nullable } from "@xrf/types";
import { EditorPopoverGroup, EditorPopoverGroupSection } from "@xrf/ui/editor/popover-group";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { CheckboxFormRow } from "@xrf/ui/form/checkbox-row";
import { Inventory2Icon } from "@xrf/ui/icons/inventory2";
import { ReactElement } from "react";

import { ILevelHelperOptions, isLevelHelperOn } from "@/core/level/lib/objects/level-helper-options";
import { getLevelVisibleOption, TLevelVisibleOption } from "@/core/level/lib/objects/level-object-visibility";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { LevelViewService } from "@/core/level/services/level-view";

/** Why a helper switch cannot be turned while helpers are off. */
const HELPERS_OFF: string = "Helpers are off: turn them on with G";

/**
 * What of the level's objects is shown, a row a kind from the kind table: a kind drawn as a model by its picture, its
 * helper beneath; a kind drawn as nothing by its helper alone; and whether what a new game releases is drawn too.
 */
export function LevelShowAction({
  "data-testid": dataTestId = "level-show-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const viewService: LevelViewService = useInjection(LevelViewService);
  const options: ILevelViewOptions = viewService.options;
  const helpers: ILevelHelperOptions = viewService.helpers;

  const shown: Array<string> = LEVEL_OBJECT_KINDS.filter((entry: LevelObjectKindEntry) => {
    const option: Nullable<TLevelVisibleOption> = getLevelVisibleOption(entry.kind);

    return option ? options[option] : false;
  }).map((entry: LevelObjectKindEntry) => entry.label.toLowerCase());

  return (
    <EditorPopoverGroup
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Show"}
      description={
        shown.length
          ? `Spawned ${shown.join(", ")}${options.isSpawnedReleased ? ", released too" : ""}`
          : "Nothing spawned drawn"
      }
      icon={<Inventory2Icon />}
      isActive={shown.length > 0}
    >
      {LEVEL_OBJECT_KINDS.map((entry: LevelObjectKindEntry) => {
        const option: Nullable<TLevelVisibleOption> = getLevelVisibleOption(entry.kind);
        const isHelped: boolean = isLevelHelperOn(helpers, entry.kind);

        return option ? (
          <EditorPopoverGroupSection
            key={entry.kind}
            label={entry.label}
            isOn={options[option]}
            onToggle={() => viewService.toggleOption(option)}
          >
            <CheckboxFormRow
              label={"Helper"}
              description={helpers.isShown ? `Outlines the ${entry.label.toLowerCase()} by kind` : HELPERS_OFF}
              isChecked={isHelped}
              isDisabled={!helpers.isShown}
              onChange={() => viewService.setHelperKinds([entry.kind], !isHelped)}
            />
          </EditorPopoverGroupSection>
        ) : (
          <EditorPopoverGroupSection
            key={entry.kind}
            label={`${entry.label} helpers`}
            description={helpers.isShown ? undefined : HELPERS_OFF}
            isOn={isHelped}
            isDisabled={!helpers.isShown}
            onToggle={() => viewService.setHelperKinds([entry.kind], !isHelped)}
          />
        );
      })}

      <EditorPopoverGroupSection
        label={"Released"}
        description={"Draws what a new game releases before the player arrives, with its group"}
        isOn={options.isSpawnedReleased}
        onToggle={() => viewService.toggleOption("isSpawnedReleased")}
      />
    </EditorPopoverGroup>
  );
}
