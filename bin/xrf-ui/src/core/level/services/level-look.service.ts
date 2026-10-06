import { inject, Injectable } from "@wirestate/core";
import { BoundAction, Computed, RefObservable, runInAction } from "@wirestate/mobx";
import { Nullable } from "@xrf/types";

import { levelsCommands } from "@/core/ipc/commands/levels";
import { LevelConsoleDefaults, SelectedLevelDescription, SessionSnapshot } from "@/core/ipc/types/xrf-app";
import { XrayEngine } from "@/core/ipc/types/xrf-engine-target";
import {
  ELevelLookSource,
  ILevelLook,
  ILevelLookChoice,
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
 * How a level is exposed, lit and corrected: the game's console defaults, the settings', a built-in engine's, or a
 * look of the viewer's own, edited by hand. The choice is kept over runs.
 */
@Injectable()
export class LevelLookService {
  /** Where the look comes from, and the custom one's values. */
  @RefObservable()
  public choice: ILevelLookChoice;

  /** What the open level's game ships as its console defaults, or null before they are read or with no level open. */
  @RefObservable()
  public defaults: Nullable<LevelConsoleDefaults> = null;

  private readonly log: Logger = new Logger(__MODULE_NAME__);
  /** The engine the open level is read as, which a look the game leaves to its engine follows; null with none open. */
  @RefObservable()
  public engine: Nullable<XrayEngine> = null;

  /** The level whose game's defaults are held or being read. */
  private sessionId: Nullable<string> = null;

  public constructor(private readonly settingsService: SettingsService = inject(SettingsService)) {
    this.choice = toLevelLookChoice(parseLocalStorageValueSafe(LEVEL_LOOK_STORAGE_KEY), this.exposure);
  }

  /**
   * The look the open level's game ships, its engine's own where the game leaves a command to it; null before its
   * defaults are read or with none open.
   */
  @Computed()
  public get game(): Nullable<ILevelLook> {
    return this.defaults && this.engine ? toGameLevelLook(this.defaults, this.exposure, this.engine) : null;
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
      this.engine = selected?.value.engine.engine ?? null;
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
   * @param source - Where the look comes from; the values edited by hand only once some were, since editing is what
   *   makes them.
   */
  @BoundAction()
  public setSource(source: ELevelLookSource): void {
    if (source !== ELevelLookSource.CUSTOM || this.choice.custom) {
      this.store({ ...this.choice, source });
    }
  }

  /**
   * Draws with values edited by hand, from whatever the look was.
   *
   * @param look - The values.
   */
  @BoundAction()
  public edit(look: ILevelLook): void {
    this.store({ custom: look, source: ELevelLookSource.CUSTOM });
  }

  private get exposure(): TRenderExposureSettings {
    return this.settingsService.rendererFeatures.exposure;
  }

  private store(choice: ILevelLookChoice): void {
    this.choice = choice;
    setLocalStorageValueSafe(LEVEL_LOOK_STORAGE_KEY, JSON.stringify(choice));
  }
}
