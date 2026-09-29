import { ListItemText, MenuItem, TextField } from "@mui/material";
import { ChangeEvent, ReactElement } from "react";

import { ThunderboltCollection } from "@/core/ipc/types/xrf-environment";
import { CONTROL } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";

/** What the select holds for a keyframe that strikes with nothing. */
const NONE: string = "";

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
    <TextField
      data-testid={dataTestId}
      id={id}
      className={className}
      select
      fullWidth
      size={"small"}
      label={"thunderbolt_collection"}
      value={value}
      disabled={!collections.length && !value}
      slotProps={{
        inputLabel: { shrink: true },
        select: {
          displayEmpty: true,
          MenuProps: { slotProps: { paper: { sx: { maxHeight: CONTROL.selectMenuMaxHeight } } } },
          renderValue: (selected: unknown) => String(selected) || "None",
        },
      }}
      onChange={(event: ChangeEvent<HTMLInputElement>) => onChange(event.target.value)}
    >
      <MenuItem dense value={NONE}>
        None
      </MenuItem>
      {/* A collection the game no longer has, kept so the keyframe still says what it names. */}
      {value && !collections.some((it: ThunderboltCollection) => it.name === value) ? (
        <MenuItem dense value={value}>
          <ListItemText primary={value} secondary={"not in the game"} />
        </MenuItem>
      ) : null}
      {collections.map((collection: ThunderboltCollection) => (
        <MenuItem key={collection.name} dense value={collection.name}>
          <ListItemText primary={collection.name} secondary={`${collection.thunderbolts.length} bolts`} />
        </MenuItem>
      ))}
    </TextField>
  );
}
