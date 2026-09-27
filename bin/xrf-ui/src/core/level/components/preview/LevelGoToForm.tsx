import { Button, TextField, Typography } from "@mui/material";
import { Nullable } from "@xrf/types";
import { ChangeEvent, FormEvent, ReactElement, useCallback, useState } from "react";

import { ILevelCamera } from "@/core/level/lib/camera/level-camera";
import {
  ILevelGoTo,
  isLevelGoToText,
  LEVEL_GO_TO_FIELDS,
  LEVEL_GO_TO_ORIGIN,
  parseLevelGoTo,
  parseLevelGoToTexts,
  toLevelGoTo,
  toLevelGoToTexts,
} from "@/core/level/lib/camera/level-camera-goto";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelGoToFormProps extends BaseComponentProps {
  /** Where the camera is as the form mounts, which the fields start at; null before the viewport has drawn. */
  readCamera: () => Nullable<ILevelCamera>;
  onGoTo: (goTo: ILevelGoTo) => void;
}

/**
 * The fields of a place to go to, which start where the camera was as the form mounted.
 */
export function LevelGoToForm({
  "data-testid": dataTestId = "level-go-to-form",
  id,
  className,
  readCamera,
  onGoTo,
}: ILevelGoToFormProps): ReactElement {
  const [start] = useState<ILevelGoTo>(() => {
    const camera: Nullable<ILevelCamera> = readCamera();

    return camera ? toLevelGoTo(camera) : LEVEL_GO_TO_ORIGIN;
  });
  const [texts, setTexts] = useState<Record<keyof ILevelGoTo, string>>(() => toLevelGoToTexts(start));
  const parsed: Nullable<ILevelGoTo> = parseLevelGoToTexts(texts);

  const onPaste = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      const pasted: Nullable<ILevelGoTo> = parseLevelGoTo(event.target.value, parsed ?? start);

      if (pasted) {
        setTexts(toLevelGoToTexts(pasted));
      }
    },
    [parsed, start]
  );

  const onSubmit = useCallback(
    (event: FormEvent) => {
      event.preventDefault();

      if (parsed) {
        onGoTo(parsed);
      }
    },
    [onGoTo, parsed]
  );

  return (
    <form
      data-testid={dataTestId}
      id={id}
      className={cn("flex w-96 flex-col gap-1 px-4 py-2", className)}
      onSubmit={onSubmit}
    >
      <Typography className={"text-text-secondary"} variant={"overline"}>
        Go to
      </Typography>

      <TextField
        size={"small"}
        margin={"dense"}
        label={"Paste the readout"}
        placeholder={"x 0 y 0 z 0 h 0 p 0"}
        onChange={onPaste}
      />

      <div className={"grid grid-cols-3 gap-x-2"}>
        {LEVEL_GO_TO_FIELDS.map(([key, label]) => (
          <TextField
            key={key}
            size={"small"}
            margin={"dense"}
            type={"number"}
            label={label}
            value={texts[key]}
            error={!isLevelGoToText(texts[key])}
            slotProps={{ htmlInput: { "aria-label": label, step: "any" } }}
            onChange={(event: ChangeEvent<HTMLInputElement>) =>
              setTexts((current) => ({ ...current, [key]: event.target.value }))
            }
          />
        ))}
      </div>

      <Button type={"submit"} size={"small"} disabled={!parsed}>
        Go
      </Button>
    </form>
  );
}
