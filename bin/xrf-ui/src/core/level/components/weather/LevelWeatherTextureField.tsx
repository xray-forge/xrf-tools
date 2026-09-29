import { Autocomplete, TextField } from "@mui/material";
import { Nullable } from "@xrf/types";
import { ReactElement, SyntheticEvent, useCallback } from "react";

import { LevelWeatherTexture } from "@/core/ipc/types/xrf-app";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelWeatherTextureFieldProps extends BaseComponentProps {
  /** The engine key it holds. */
  label: string;
  /** The reference, empty for none. */
  value: string;
  /** Every texture of its kind the game's weather names. */
  textures: ReadonlyArray<LevelWeatherTexture>;
  onChange: (reference: string) => void;
}

/**
 * A weather texture key: one the game's weather names, with how many keyframes use it and whether it resolves, or any
 * reference typed.
 */
export function LevelWeatherTextureField({
  "data-testid": dataTestId = "level-weather-texture-field",
  id,
  className,
  label,
  value,
  textures,
  onChange,
}: ILevelWeatherTextureFieldProps): ReactElement {
  const onPick = useCallback(
    (_: SyntheticEvent, next: Nullable<string | LevelWeatherTexture>) => {
      const reference: string = typeof next === "string" ? next.trim() : (next?.texture.reference ?? "");

      if (reference !== value) {
        onChange(reference);
      }
    },
    [value, onChange]
  );

  return (
    <Autocomplete<LevelWeatherTexture, false, false, true>
      data-testid={dataTestId}
      id={id}
      className={className}
      freeSolo
      autoSelect
      size={"small"}
      options={textures as Array<LevelWeatherTexture>}
      value={value}
      getOptionLabel={(option: string | LevelWeatherTexture) =>
        typeof option === "string" ? option : option.texture.reference
      }
      renderOption={({ key, ...props }, option: LevelWeatherTexture) => (
        <li key={key} {...props}>
          <div className={"flex w-full min-w-0 items-baseline justify-between gap-2"}>
            <span className={"min-w-0 truncate font-mono text-xs"}>{option.texture.reference}</span>
            <span className={option.texture.logicalPath ? "text-xs text-text-secondary" : "text-xs text-warning"}>
              {option.texture.logicalPath ? `${option.uses}×` : "missing"}
            </span>
          </div>
        </li>
      )}
      renderInput={(params) => <TextField {...params} label={label} />}
      onChange={onPick}
    />
  );
}
