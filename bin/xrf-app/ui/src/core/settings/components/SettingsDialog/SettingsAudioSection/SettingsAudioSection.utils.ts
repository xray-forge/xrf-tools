import {
  AudioDeviceDescription,
  AudioOutputChoice,
  AudioOutputStatus,
  EAudioOutputChoice,
} from "@xrf/ipc/types/xrf-audio";
import { Nullable } from "@xrf/types";
import { IChoiceFormRowOption } from "@xrf/ui/form/choice-row";

/** An output choice as one string a choice row can hold. */
export type TAudioOutputKey = string;

const SYSTEM_DEFAULT_KEY: TAudioOutputKey = EAudioOutputChoice.SYSTEM_DEFAULT;

const DEVICE_KEY_PREFIX: string = "device:";

/**
 * @param choice - Which device sound plays on.
 * @returns The choice as the key the row holds.
 */
export function toAudioOutputKey(choice: AudioOutputChoice): TAudioOutputKey {
  return choice.kind === EAudioOutputChoice.DEVICE ? `${DEVICE_KEY_PREFIX}${choice.id}` : SYSTEM_DEFAULT_KEY;
}

/**
 * @param key - A key the row held.
 * @returns The output choice it names.
 */
export function fromAudioOutputKey(key: TAudioOutputKey): AudioOutputChoice {
  return key.startsWith(DEVICE_KEY_PREFIX)
    ? { kind: EAudioOutputChoice.DEVICE, id: key.slice(DEVICE_KEY_PREFIX.length) }
    : { kind: EAudioOutputChoice.SYSTEM_DEFAULT };
}

/**
 * The devices a person can pick: the system default, every device the system offers, and the chosen one while it is
 * missing, so the choice never disappears from under the row.
 *
 * @param devices - The devices the system offers, or null before it was asked.
 * @param choice - The device the settings name.
 * @returns The row's options.
 */
export function listAudioOutputOptions(
  devices: Nullable<ReadonlyArray<AudioDeviceDescription>>,
  choice: AudioOutputChoice
): ReadonlyArray<IChoiceFormRowOption<TAudioOutputKey>> {
  const options: Array<IChoiceFormRowOption<TAudioOutputKey>> = [
    { label: "System default", value: SYSTEM_DEFAULT_KEY },
    ...(devices ?? []).map((device: AudioDeviceDescription) => ({
      label: device.name,
      value: toAudioOutputKey({ kind: EAudioOutputChoice.DEVICE, id: device.id }),
    })),
  ];
  const chosen: TAudioOutputKey = toAudioOutputKey(choice);

  if (!options.some((option: IChoiceFormRowOption<TAudioOutputKey>) => option.value === chosen)) {
    options.push({ label: "Missing device", value: chosen });
  }

  return options;
}

/**
 * @param output - What the output plays on, or null before it was asked.
 * @returns What the device row says beneath its choice.
 */
export function describeAudioOutput(output: Nullable<AudioOutputStatus>): string {
  const lead: string = "The device sound plays on. The system default follows the system as it changes.";

  if (!output) {
    return lead;
  }

  if (output.failure) {
    return `${lead} No device plays: ${output.failure}`;
  }

  const playing: string = `Playing on ${output.deviceName ?? "an unnamed device"} at ${output.sampleRate} Hz.`;

  return output.isFallback
    ? `${lead} The chosen device is missing, so the system default plays. ${playing}`
    : `${lead} ${playing}`;
}
