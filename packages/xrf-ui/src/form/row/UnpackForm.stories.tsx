import type { Meta, StoryObj } from "@storybook/react-vite";
import { Nullable } from "@xrf/types";
import { ChangeEvent, ReactElement, useCallback, useState } from "react";

import { Button } from "#/button";
import { CheckboxFormRow } from "#/form/checkbox-row";
import { ChoiceFormRow, IChoiceFormRowOption } from "#/form/choice-row";
import { SliderFormRow } from "#/form/slider-row";
import { SwitchFormRow } from "#/form/switch-row";
import { TextField } from "#/input";
import { Card } from "#/layout/card";
import { FORM_SURFACE_SX } from "#/theme/sx";
import { Typography } from "#/typography";

import { FormRow } from "./FormRow";

type TSoundFormat = "ogg" | "wav";

const SOUND_FORMAT_OPTIONS: ReadonlyArray<IChoiceFormRowOption<TSoundFormat>> = [
  { value: "ogg", label: "Keep ogg" },
  { value: "wav", label: "Decode to wav" },
];

function UnpackFormPattern(): ReactElement {
  const [directory, setDirectory] = useState<string>("gamedata");
  const [soundFormat, setSoundFormat] = useState<TSoundFormat>("ogg");
  const [threads, setThreads] = useState<number>(8);
  const [isVerifying, setVerifying] = useState<boolean>(true);
  const [isOverwriting, setOverwriting] = useState<boolean>(false);
  const directoryError: Nullable<string> = directory.trim() ? null : "Name the output directory.";

  const onDirectoryChange = useCallback((event: ChangeEvent<HTMLInputElement>) => setDirectory(event.target.value), []);

  return (
    <Card className={"flex max-w-reading flex-col gap-5 p-6"} sx={FORM_SURFACE_SX}>
      <div>
        <Typography variant={"h6"}>Unpack archives</Typography>
        <Typography className={"text-text-secondary"} variant={"body2"}>
          Writes every entry of the selected archives under one directory.
        </Typography>
      </div>

      <FormRow
        label={"Output directory"}
        description={"Created beside the archives when it does not exist."}
        error={directoryError}
        fact={directoryError ? null : `Writes into ${directory}\\`}
      >
        {({ id, ...aria }) => (
          <TextField
            id={id}
            fullWidth
            value={directory}
            error={Boolean(directoryError)}
            slotProps={{ htmlInput: aria }}
            onChange={onDirectoryChange}
          />
        )}
      </FormRow>

      <ChoiceFormRow<TSoundFormat>
        label={"Sound format"}
        options={SOUND_FORMAT_OPTIONS}
        value={soundFormat}
        onChange={setSoundFormat}
      />

      <SliderFormRow
        label={"Threads"}
        description={"How many archives are read at once."}
        value={threads}
        min={1}
        max={32}
        step={1}
        recommended={[4, 16]}
        onChange={setThreads}
      />

      <CheckboxFormRow
        label={"Verify checksums"}
        description={"Reads each entry back after writing it."}
        isChecked={isVerifying}
        onChange={setVerifying}
      />

      <SwitchFormRow
        label={"Overwrite existing files"}
        isChecked={isOverwriting}
        isRequired={false}
        onChange={setOverwriting}
      />

      <div className={"flex justify-end gap-2"}>
        <Button>Cancel</Button>
        <Button variant={"contained"} disabled={Boolean(directoryError)}>
          Unpack
        </Button>
      </div>
    </Card>
  );
}

const meta = {
  title: "Patterns/Form",
  component: UnpackFormPattern,
} satisfies Meta<typeof UnpackFormPattern>;

export default meta;

type TStory = StoryObj<typeof meta>;

/**
 * A form somebody sits down and fills, composed from the form rows on a card that applies `FORM_SURFACE_SX`: controls
 * take the form's height and corner, inline rows are drawn as bordered wells. Clear the directory to see the error.
 */
export const Unpack: TStory = {};
