import { useInjection } from "@wirestate/react";
import { LEVEL_OBJECT_KINDS, LevelObjectKindEntry } from "@xrf/ipc/types/xrf-app-levels";
import { Nullable } from "@xrf/types";
import { EditorPopoverGroupSection } from "@xrf/ui/editor/popover-group";
import {
  EditorPopoverCheckboxTable,
  EditorPopoverSwitchRow,
  IEditorPopoverCheckboxColumn,
  IEditorPopoverCheckboxTableRow,
} from "@xrf/ui/editor/popover-row";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ViewInArIcon } from "@xrf/ui/icons/view-in-ar";
import { VisibilityIcon } from "@xrf/ui/icons/visibility";
import { ReactElement } from "react";

import {
  ILevelHelperOptions,
  isLevelHelperOn,
  LEVEL_HELPERS_OFF_REASON,
} from "@/core/level/lib/objects/level-helper-options";
import { getLevelVisibleOption, TLevelVisibleOption } from "@/core/level/lib/objects/level-object-visibility";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { LevelViewService } from "@/core/level/services/level-view";

/** The kind table's two switches: whether a kind's objects are drawn, and whether its helper marks them. */
const COLUMNS: ReadonlyArray<IEditorPopoverCheckboxColumn> = [
  { icon: <VisibilityIcon />, id: "shown", label: "Shown" },
  { icon: <ViewInArIcon />, id: "helper", label: "Helper" },
];

/**
 * Which of the level's objects are drawn and helped, a row a kind of the kind table: a kind drawn as a model shown or
 * not, every kind's helper, which waits on the helpers being on; then whether what a new game releases is drawn too.
 */
export function LevelShowObjects({
  "data-testid": dataTestId = "level-show-objects",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const viewService: LevelViewService = useInjection(LevelViewService);
  const options: ILevelViewOptions = viewService.options;
  const helpers: ILevelHelperOptions = viewService.helpers;

  const rows: Array<IEditorPopoverCheckboxTableRow> = LEVEL_OBJECT_KINDS.map((entry: LevelObjectKindEntry) => {
    const option: Nullable<TLevelVisibleOption> = getLevelVisibleOption(entry.kind);
    const isHelped: boolean = isLevelHelperOn(helpers, entry.kind);

    return {
      cells: [
        option ? { isChecked: options[option], onChange: () => viewService.toggleOption(option) } : null,
        {
          isChecked: isHelped,
          isDisabled: !helpers.isShown,
          onChange: () => viewService.setHelperKinds({ isOn: !isHelped, kinds: [entry.kind] }),
          reason: LEVEL_HELPERS_OFF_REASON,
        },
      ],
      id: entry.kind,
      label: entry.label,
    };
  });

  return (
    <EditorPopoverGroupSection data-testid={dataTestId} id={id} className={className} label={"Objects"}>
      <EditorPopoverCheckboxTable columns={COLUMNS} rows={rows} />

      <EditorPopoverSwitchRow
        className={"mt-1"}
        label={"Released"}
        description={"Draws what a new game releases before the player arrives, with its group."}
        isChecked={options.isSpawnedReleased}
        onChange={() => viewService.toggleOption("isSpawnedReleased")}
      />
    </EditorPopoverGroupSection>
  );
}
