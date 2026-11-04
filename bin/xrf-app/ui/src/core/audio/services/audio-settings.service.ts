import { Injectable, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, RefObservable, runInAction } from "@wirestate/mobx";
import { audioCommands } from "@xrf/ipc/commands/audio";
import { AudioSettings } from "@xrf/ipc/types/xrf-app-audio";
import {
  AudioDeviceDescription,
  AudioOutputChoice,
  AudioOutputStatus,
  AudioSpatialization,
} from "@xrf/ipc/types/xrf-audio";
import { Logger } from "@xrf/logging";
import { saturate } from "@xrf/math";
import { Nullable } from "@xrf/types";

import { readAudioVolume, TAudioVolume } from "@/core/audio/lib/audio-volumes";

/**
 * How the application plays sound: the output device, the volumes, the background mute, and whether placed sounds are
 * heard for headphones or speakers. The backend keeps them between runs, so they hold before any page loads; this
 * reads them once and sends each change, taking back what the backend kept.
 */
@Injectable()
export class AudioSettingsService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** The settings the backend plays with, or null before they are read. */
  @RefObservable()
  public settings: Nullable<AudioSettings> = null;

  /** The output devices the system offered when last asked, or null before it was. */
  @RefObservable()
  public devices: Nullable<ReadonlyArray<AudioDeviceDescription>> = null;

  /** What the output played on when last asked, or null before it was. */
  @RefObservable()
  public output: Nullable<AudioOutputStatus> = null;

  /** How many writes went out, so only the latest one's answer is taken back. */
  private writes: number = 0;

  /** How loud previews play, in `[0, 1]`. */
  @Computed()
  public get previewsVolume(): number {
    return this.getVolume("previews");
  }

  /**
   * @param volume - The category, or the master.
   * @returns Its volume in `[0, 1]`, full before the settings are read.
   */
  public getVolume(volume: TAudioVolume): number {
    return readAudioVolume(this.settings?.volumes[volume]);
  }

  @OnProvision()
  public onProvision(): void {
    void this.read();
  }

  /** Reads the settings again. */
  @BoundAction()
  public async read(): Promise<void> {
    try {
      const settings: AudioSettings = await audioCommands.describeSettings();

      runInAction(() => (this.settings = settings));
    } catch (error: unknown) {
      this.log.warn("Could not read the audio settings:", error);
    }
  }

  /** Asks the system for its output devices and what the output plays on now, for the settings to show. */
  @BoundAction()
  public async readDevices(): Promise<void> {
    try {
      const [devices, output] = await Promise.all([audioCommands.listDevices(), audioCommands.describeOutput()]);

      runInAction(() => {
        this.devices = devices;
        this.output = output;
      });
    } catch (error: unknown) {
      this.log.warn("Could not list the audio devices:", error);
    }
  }

  /**
   * @param volume - The category, or the master.
   * @param value - Its new volume, held in `[0, 1]`.
   */
  @BoundAction()
  public setVolume(volume: TAudioVolume, value: number): void {
    if (this.settings) {
      void this.write({
        ...this.settings,
        volumes: { ...this.settings.volumes, [volume]: saturate(value) },
      });
    }
  }

  @BoundAction()
  public setPreviewsVolume(value: number): void {
    this.setVolume("previews", value);
  }

  @BoundAction()
  public setOutput(output: AudioOutputChoice): void {
    if (this.settings) {
      this.log.info("Set audio output:", output);

      void this.write({ ...this.settings, output }).then(() => this.readDevices());
    }
  }

  @BoundAction()
  public setMutedInBackground(isMutedInBackground: boolean): void {
    if (this.settings) {
      void this.write({ ...this.settings, isMutedInBackground });
    }
  }

  @BoundAction()
  public setSpatialization(spatialization: AudioSpatialization): void {
    if (this.settings) {
      void this.write({ ...this.settings, spatialization });
    }
  }

  /**
   * Shows the settings at once and takes back what the backend kept of them, unless a later write has gone out since:
   * a slider dragged sends several, and an earlier answer must not pull it back.
   */
  private async write(settings: AudioSettings): Promise<void> {
    const write: number = ++this.writes;

    this.settings = settings;

    try {
      const kept: AudioSettings = await audioCommands.setSettings(settings);

      if (write === this.writes) {
        runInAction(() => (this.settings = kept));
      }
    } catch (error: unknown) {
      this.log.warn("The audio settings were not kept:", error);
    }
  }
}
