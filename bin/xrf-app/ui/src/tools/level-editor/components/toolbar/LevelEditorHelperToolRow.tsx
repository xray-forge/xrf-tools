import { formatCount } from "@xrf/format";
import { SceneHelperToolEntry } from "@xrf/ipc/types/xrf-app-levels";
import { RenderHelperIcon } from "@xrf/ipc/types/xrf-renderer";
import { Nullable } from "@xrf/types";
import { EditorPopoverSwitchRow } from "@xrf/ui/editor/popover-row";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { cn } from "@xrf/ui/theme";
import { ReactElement } from "react";

import { LEVEL_KIND_ICONS } from "@/core/level/components/objects/LevelKindIcon";
import { SCENE_TOOL_LABELS } from "@/tools/level-editor/lib/tools/scene-tools";

interface ILevelEditorHelperToolRowProps extends BaseComponentProps {
  /** The helped tool's row of the helper table. */
  entry: SceneHelperToolEntry;
  /** How many of the open scene's objects of the tool draw a helper. */
  count: number;
  isOn: boolean;
  isDisabled?: boolean;
  /** Why it cannot be switched, while it is disabled. */
  reason?: string;
  onChange: (isOn: boolean) => void;
}

/**
 * One helped tool's switch in the Show action, with its helper's icon or colour swatch and the scene's helper count.
 */
export function LevelEditorHelperToolRow({
  "data-testid": dataTestId = "level-editor-helper-tool-row",
  id,
  className,
  entry,
  count,
  isOn,
  isDisabled = false,
  reason,
  onChange,
}: ILevelEditorHelperToolRowProps): ReactElement {
  const Icon: Nullable<(typeof LEVEL_KIND_ICONS)[RenderHelperIcon]> = entry.icon ? LEVEL_KIND_ICONS[entry.icon] : null;
  const label: string = SCENE_TOOL_LABELS[entry.tool];

  return (
    <div data-testid={dataTestId} id={id} className={cn("flex items-center gap-1.5", className)}>
      <EditorPopoverSwitchRow
        className={"min-w-0 flex-1"}
        label={label}
        description={isDisabled ? reason : `Draws the helpers of the scene's ${label.toLowerCase()}.`}
        isChecked={isOn}
        isDisabled={isDisabled}
        onChange={onChange}
      />

      {Icon ? (
        <Icon className={"shrink-0"} sx={{ color: entry.color, fontSize: 14 }} aria-hidden />
      ) : (
        <span aria-hidden className={"size-2.5 shrink-0 rounded-control"} style={{ backgroundColor: entry.color }} />
      )}

      <span className={"w-10 shrink-0 text-right text-xs text-text-secondary tabular-nums"}>{formatCount(count)}</span>
    </div>
  );
}
