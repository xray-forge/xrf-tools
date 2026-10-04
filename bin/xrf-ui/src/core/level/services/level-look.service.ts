import { inject, Injectable } from "@wirestate/core";
import { BoundAction, Computed, RefObservable, runInAction } from "@wirestate/mobx";
import { Nullable } from "@xrf/types";

import { levelsCommands } from "@/core/ipc/commands/levels";
import { LevelConsoleDefaults, SelectedLevelDescription, SessionSnapshot } from "@/core/ipc/types/xrf-app";
import {
  DEFAULT_LEVEL_LOOK_CHOICE,
  ELevelLookSource,
  ILevelLook,
  ILevelLookChoice,
  ILevelLookPreset,
  resolveLevelLook,
  toGameLevelLook,
  toLevelLookChoice,
} from "@/core/level/lib/look";
import { TRenderExposureSettings } from "@/core/render/lib/settings/render-feature-settings";
import { SettingsService } from "@/core/settings/services/settings";
import { LEVEL_LOOK_STORAGE_KEY } from "@/core/storage";
import { parseLocalStorageValueSafe, setLocalStorageValueSafe } from "@/lib/local-storage";
import { Logger } from "@/lib/logging";

/**
 * How a level is exposed, lit and corrected: the game's console defaults, the settings', or a look of the viewer's
 * own, picked from the presets it saved or edited by hand. The choice and the presets are kept over runs.
 */
@Injectable()
export class LevelLookService {
  /** Where the look comes from, the custom one's values, and the presets saved. */
  @RefObservable()
  public choice: ILevelLookChoice;

  /** What the open level's game ships as its console defaults, or null before they are read or with no level open. */
  @RefObservable()
  public defaults: Nullable<LevelConsoleDefaults> = null;

  private readonly log: Logger = new Logger("LevelLookService");
  /** The level whose game's defaults are held or being read. */
  private sessionId: Nullable<string> = null;

  public constructor(private readonly settingsService: SettingsService = inject(SettingsService)) {
    this.choice = toLevelLookChoice(parseLocalStorageValueSafe(LEVEL_LOOK_STORAGE_KEY), this.exposure);
  }

  /** The look the open level's game ships, or null where it ships no console defaults or none is open. */
  @Computed()
  public get game(): Nullable<ILevelLook> {
    return this.defaults?.isShipped ? toGameLevelLook(this.defaults, this.exposure) : null;
  }

  /** The look the level is drawn with. */
  @Computed()
  public get look(): ILevelLook {
    return resolveLevelLook(this.choice, this.game, this.exposure);
  }

  /**
   * Reads the console defaults of a level's game, once per level.
   *
   * @param selected - The level open now, or null for none.
   */
  public async open(selected: Nullable<SessionSnapshot<SelectedLevelDescription>>): Promise<void> {
    const sessionId: Nullable<string> = selected?.sessionId ?? null;

    if (sessionId === this.sessionId) {
      return;
    }

    this.sessionId = sessionId;
    runInAction(() => {
      this.defaults = null;
    });

    if (!sessionId) {
      return;
    }

    try {
      const { value } = await levelsCommands.describeConsoleDefaults(sessionId);

      if (this.sessionId === sessionId) {
        runInAction(() => {
          this.defaults = value;
        });
      }
    } catch (error: unknown) {
      this.log.warn("The game's console defaults cannot be read, so its levels look as the settings say:", error);
    }
  }

  /**
   * @param source - A look of its own source; a custom one is chosen by editing or by picking a preset.
   */
  @BoundAction()
  public setSource(source: Exclude<ELevelLookSource, ELevelLookSource.CUSTOM>): void {
    this.store({ ...this.choice, preset: null, source });
  }

  /**
   * Draws with a saved preset's values, as a custom look taken from it.
   *
   * @param name - The preset's name.
   */
  @BoundAction()
  public pickPreset(name: string): void {
    const preset: ILevelLookPreset | undefined = this.choice.presets.find((it: ILevelLookPreset) => it.name === name);

    if (preset) {
      this.store({ ...this.choice, custom: preset.look, preset: name, source: ELevelLookSource.CUSTOM });
    }
  }

  /**
   * Draws with values edited by hand, from whatever the look was.
   *
   * @param look - The values.
   */
  @BoundAction()
  public edit(look: ILevelLook): void {
    this.store({ ...this.choice, custom: look, preset: null, source: ELevelLookSource.CUSTOM });
  }

  /**
   * Saves the look drawn now under a name, replacing a preset of that name.
   *
   * @param name - The name, trimmed; an empty one saves nothing.
   */
  @BoundAction()
  public savePreset(name: string): void {
    const trimmed: string = name.trim();

    if (!trimmed) {
      return;
    }

    const presets: Array<ILevelLookPreset> = [
      ...this.choice.presets.filter((it: ILevelLookPreset) => it.name !== trimmed),
      { look: this.look, name: trimmed },
    ].sort((a: ILevelLookPreset, b: ILevelLookPreset) => a.name.localeCompare(b.name));

    this.store({ ...this.choice, custom: this.look, preset: trimmed, presets, source: ELevelLookSource.CUSTOM });
  }

  /**
   * Forgets a preset; the look it was drawing with stays, as edited by hand.
   *
   * @param name - The preset's name.
   */
  @BoundAction()
  public deletePreset(name: string): void {
    this.store({
      ...this.choice,
      preset: this.choice.preset === name ? null : this.choice.preset,
      presets: this.choice.presets.filter((it: ILevelLookPreset) => it.name !== name),
    });
  }

  /** Back to the game's own look, the presets kept. */
  @BoundAction()
  public reset(): void {
    this.store({ ...DEFAULT_LEVEL_LOOK_CHOICE, presets: this.choice.presets });
  }

  private get exposure(): TRenderExposureSettings {
    return this.settingsService.rendererFeatures.exposure;
  }

  private store(choice: ILevelLookChoice): void {
    this.choice = choice;
    setLocalStorageValueSafe(LEVEL_LOOK_STORAGE_KEY, JSON.stringify(choice));
  }
}
