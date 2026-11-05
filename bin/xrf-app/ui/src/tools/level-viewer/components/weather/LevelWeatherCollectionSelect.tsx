import { formatCount } from "@xrf/format";
import { ThunderboltCollection } from "@xrf/ipc/types/xrf-environment";
import { EditorPopoverSelectRow } from "@xrf/ui/editor/popover-row";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ListItemText } from "@xrf/ui/material/list";
import { MenuItem } from "@xrf/ui/material/menu";
import { ReactElement } from "react";

import {
  describeLevelManualWeatherKey,
  LEVEL_MANUAL_WEATHER_LABELS,
} from "@/core/level/lib/weather/level-manual-weather-labels";

/** What the select holds for a keyframe that strikes with nothing. */
const NONE: string = "";

/** How many bolts a collection strikes with. */
function formatThunderbolts(count: number): string {
  return `${formatCount(count)} ${count === 1 ? "bolt" : "bolts"}`;
}

interface ILevelWeatherCollectionSelectProps extends BaseComponentProps {
  /** The collection struck with, empty for none. */
  value: string;
  collections: ReadonlyArray<ThunderboltCollection>;
  onChange: (collection: string) => void;
}

/**
 * `thunderbolt_collection`: the game's collections, whose bolts a keyframe strikes with at random; None strikes none.
 */
export function LevelWeatherCollectionSelect({
  "data-testid": dataTestId = "level-weather-collection-select",
  id,
  className,
  value,
  collections,
  onChange,
}: ILevelWeatherCollectionSelectProps): ReactElement {
  return (
    <EditorPopoverSelectRow
      data-testid={dataTestId}
      id={id}
      className={className}
      label={LEVEL_MANUAL_WEATHER_LABELS.thunderboltCollection}
      description={describeLevelManualWeatherKey("thunderboltCollection")}
      value={value}
      isDisabled={!collections.length && !value}
      renderValue={(selected: string) => selected || "None"}
      onChange={onChange}
    >
      <MenuItem dense value={NONE}>
        None
      </MenuItem>

      {value && !collections.some((it: ThunderboltCollection) => it.name === value) ? (
        <MenuItem dense value={value}>
          <ListItemText primary={value} secondary={"not in the game"} />
        </MenuItem>
      ) : null}

      {collections.map((collection: ThunderboltCollection) => (
        <MenuItem key={collection.name} dense value={collection.name}>
          <ListItemText primary={collection.name} secondary={formatThunderbolts(collection.thunderbolts.length)} />
        </MenuItem>
      ))}
    </EditorPopoverSelectRow>
  );
}
