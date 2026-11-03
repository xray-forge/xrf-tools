import { useInjection } from "@wirestate/react";
import { formatPercent } from "@xrf/format";
import { AudioSettings } from "@xrf/ipc/types/xrf-app-audio";
import { Nullable } from "@xrf/types";
import { CheckboxFormRow } from "@xrf/ui/form/checkbox-row";
import { ChoiceListFormRow } from "@xrf/ui/form/choice-list-row";
import { SliderFormRow } from "@xrf/ui/form/slider-row";
import { DelayedProgress } from "@xrf/ui/layout/delayed-progress";
import { ReactElement } from "react";

import {
  AUDIO_VOLUME_DESCRIPTIONS,
  AUDIO_VOLUME_LABELS,
  AUDIO_VOLUMES,
  TAudioVolume,
} from "@/core/audio/lib/audio-volumes";
import { AudioSettingsService } from "@/core/audio/services/audio-settings.service";
import { useMountEffect } from "@/lib/react/use-mount-effect";

import {
  describeAudioOutput,
  fromAudioOutputKey,
  listAudioOutputOptions,
  TAudioOutputKey,
  toAudioOutputKey,
} from "./SettingsAudioSection.utils";

/** How the application plays sound: on which device, how loud each category, and whether it mutes in the background. */
export function SettingsAudioSection(): ReactElement {
  const audioSettingsService: AudioSettingsService = useInjection(AudioSettingsService);

  const settings: Nullable<AudioSettings> = audioSettingsService.settings;

  useMountEffect(() => {
    void audioSettingsService.readDevices();
  });

  if (!settings) {
    return <DelayedProgress />;
  }

  return (
    <div className={"flex flex-col gap-6"}>
      <ChoiceListFormRow
        label={"Output device"}
        description={describeAudioOutput(audioSettingsService.output)}
        options={listAudioOutputOptions(audioSettingsService.devices, settings.output)}
        value={toAudioOutputKey(settings.output)}
        onChange={(key: TAudioOutputKey) => audioSettingsService.setOutput(fromAudioOutputKey(key))}
      />

      {AUDIO_VOLUMES.map((volume: TAudioVolume) => (
        <SliderFormRow
          key={volume}
          label={AUDIO_VOLUME_LABELS[volume]}
          description={AUDIO_VOLUME_DESCRIPTIONS[volume]}
          value={audioSettingsService.getVolume(volume)}
          min={0}
          max={1}
          step={0.01}
          format={formatPercent}
          onChange={(value: number) => audioSettingsService.setVolume(volume, value)}
        />
      ))}

      <CheckboxFormRow
        label={"Mute in the background"}
        description={"Fall silent while no window of the application has the focus."}
        isChecked={settings.isMutedInBackground}
        onChange={audioSettingsService.setMutedInBackground}
      />
    </div>
  );
}
