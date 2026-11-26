import { LevelWeatherTexture } from "@xrf/ipc/types/xrf-level-source";
import { Nullable } from "@xrf/types";
import { EditorPopoverRow, IEditorPopoverRowControlProps } from "@xrf/ui/editor/popover-row";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { CheckIcon } from "@xrf/ui/icons/check";
import { Autocomplete, TextField } from "@xrf/ui/input";
import { cn } from "@xrf/ui/theme";
import { POPOVER_INPUT_SX } from "@xrf/ui/theme/sx";
import { ReactElement, SyntheticEvent, useCallback } from "react";

interface ILevelWeatherTextureFieldProps extends BaseComponentProps {
  label: string;
  /** Read on hovering its label, such as the config key it is written under. */
  hint?: string;
  /** The reference, empty for none. */
  value: string;
  /** Every texture of its kind the game's weather names. */
  textures: ReadonlyArray<LevelWeatherTexture>;
  onChange: (reference: string) => void;
}

/**
 * A weather texture key in a popover row: a texture the game's weather names, with its use count and whether it
 * resolves, or any typed reference.
 */
export function LevelWeatherTextureField({
  "data-testid": dataTestId = "level-weather-texture-field",
  id,
  className,
  label,
  hint,
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
    <EditorPopoverRow id={id} className={className} label={label} description={hint}>
      {({
        id: controlId,
        "aria-labelledby": labelId,
        "aria-describedby": descriptionId,
      }: IEditorPopoverRowControlProps) => (
        <Autocomplete<LevelWeatherTexture, false, false, true>
          data-testid={dataTestId}
          id={controlId}
          className={"min-w-0 flex-1"}
          freeSolo
          autoSelect
          size={"small"}
          options={textures as Array<LevelWeatherTexture>}
          value={value}
          getOptionLabel={(option: string | LevelWeatherTexture) =>
            typeof option === "string" ? option : option.texture.reference
          }
          // Matched by reference, so the current texture is marked and scrolled to when the list opens.
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
          renderInput={(params) => (
            <TextField
              {...params}
              slotProps={{
                ...params.slotProps,
                input: { ...params.slotProps.input, sx: POPOVER_INPUT_SX },
                htmlInput: {
                  ...params.slotProps.htmlInput,
                  className: cn(params.slotProps.htmlInput.className, "font-mono"),
                  "aria-labelledby": labelId,
                  "aria-describedby": descriptionId,
                },
              }}
            />
          )}
          onChange={onPick}
        />
      )}
    </EditorPopoverRow>
  );
}
