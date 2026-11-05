import { useInjection } from "@wirestate/react";
import { LEVEL_OBJECT_KINDS, LevelObjectKindEntry } from "@xrf/ipc/types/xrf-app-levels";
import { Nullable } from "@xrf/types";
import { EditorPopoverGroup } from "@xrf/ui/editor/popover-group";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { VisibilityIcon } from "@xrf/ui/icons/visibility";
import { ReactElement } from "react";

import { getLevelVisibleOption, TLevelVisibleOption } from "@/core/level/lib/objects/level-object-visibility";
import { ILevelOverlay, LEVEL_OVERLAYS } from "@/core/level/lib/view/level-overlays";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { LevelViewService } from "@/core/level/services/level-view";

import { LevelShowEffects } from "./LevelShowEffects";
import { LevelShowHelpers } from "./LevelShowHelpers";
import { LevelShowObjects } from "./LevelShowObjects";
import { LevelShowOverlays } from "./LevelShowOverlays";

/** The kinds a model draws, each switched on or off: the rest are drawn as nothing, their helper alone. */
const DRAWN_KINDS: ReadonlyArray<TLevelVisibleOption> = LEVEL_OBJECT_KINDS.map((entry: LevelObjectKindEntry) =>
  getLevelVisibleOption(entry.kind)
).filter((option: Nullable<TLevelVisibleOption>): option is TLevelVisibleOption => option !== null);

/**
 * What the view shows over and of the level, as an editor's show menu gathers it: the overlays, the helpers with their
 * switch (`G`), the particles and ambient effects, and a kind table of which objects are drawn and helped. The View
 * menu offers each as well.
 */
export function LevelShowAction({
  "data-testid": dataTestId = "level-show-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const viewService: LevelViewService = useInjection(LevelViewService);
  const options: ILevelViewOptions = viewService.options;
  const isHelped: boolean = viewService.helpers.isShown;

  const overlays: Array<string> = LEVEL_OVERLAYS.filter((it: ILevelOverlay) => options[it.option]).map(
    (it: ILevelOverlay) => it.label.toLowerCase()
  );
  const drawn: number = DRAWN_KINDS.filter((option: TLevelVisibleOption) => options[option]).length;
  const summary: string = [
    ...overlays,
    isHelped ? "helpers" : "no helpers",
    options.isParticled ? "particles" : "no particles",
    `${drawn} of ${DRAWN_KINDS.length} kinds drawn`,
  ].join(", ");

  return (
    <EditorPopoverGroup
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Show"}
      description={summary.charAt(0).toUpperCase() + summary.slice(1)}
      icon={<VisibilityIcon />}
      isActive={overlays.length > 0 || isHelped || options.isParticled || drawn > 0}
    >
      <LevelShowOverlays />
      <LevelShowHelpers />
      <LevelShowEffects />
      <LevelShowObjects />
    </EditorPopoverGroup>
  );
}
