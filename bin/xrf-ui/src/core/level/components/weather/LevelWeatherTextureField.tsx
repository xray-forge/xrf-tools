import { default as CheckIcon } from "@mui/icons-material/Check";
import { Autocomplete, TextField } from "@mui/material";
import { Nullable } from "@xrf/types";
import { ReactElement, SyntheticEvent, useCallback } from "react";

import { LevelWeatherTexture } from "@/core/ipc/types/xrf-app";
import { cn } from "@/lib/dom/dom-name";
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
      // The value is a reference and the options are textures: matched by reference, the one drawn now is marked and
      // scrolled to when the list opens.
      isOptionEqualToValue={(option: LevelWeatherTexture, current: string | LevelWeatherTexture) =>
        option.texture.reference === (typeof current === "string" ? current : current.texture.reference)
      }
      renderOption={({ key, ...props }, option: LevelWeatherTexture, { selected }) => (
        <li key={key} {...props}>
          <div className={"flex w-full min-w-0 items-center justify-between gap-2"}>
            <CheckIcon
              className={cn("shrink-0 text-primary", selected ? "visible" : "invisible")}
              fontSize={"inherit"}
            />
            <span className={cn("min-w-0 grow truncate font-mono text-xs", selected ? "text-primary" : null)}>
              {option.texture.reference}
            </span>
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
