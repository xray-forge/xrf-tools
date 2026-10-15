import { inject, Injectable, OnDeprovision, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, RefObservable, runInAction } from "@wirestate/mobx";
import { AsyncState } from "@xrf/async";
import { Nullable } from "@xrf/types";

import { transformError } from "@/core/error/lib/transform-error";
import { levelsCommands } from "@/core/ipc/commands/levels";
import { LevelConsoleDefaults, SelectedLevelDescription, SessionSnapshot } from "@/core/ipc/types/xrf-app";
import { XrayEngine } from "@/core/ipc/types/xrf-engine-target";
import { ILevelLook, toGameLevelLook } from "@/core/level/lib/look/level-look";
import {
  ELevelLookSource,
  ILevelLookChoice,
  resolveLevelLook,
  toLevelLookChoice,
} from "@/core/level/lib/look/level-look-choice";
import { LevelLoadService } from "@/core/level/services/level-load.service";
import { TRenderExposureSettings } from "@/core/render/lib/settings/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings";
import { LEVEL_LOOK_STORAGE_KEY, parseLocalStorageValueSafe, setLocalStorageValueSafe } from "@/core/storage";
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

  /** What the open level's game ships as its console defaults: idle with no level open, then read, or failed to. */
  @RefObservable()
  public defaults: AsyncState<LevelConsoleDefaults> = AsyncState.idle();

  private readonly log: Logger = new Logger(__MODULE_NAME__);
  /** The engine the open level is read as, which a look the game leaves to its engine follows; null with none open. */
  @RefObservable()
  public engine: Nullable<XrayEngine> = null;

  /** The level whose game's defaults are held or being read. */
  private sessionId: Nullable<string> = null;

  /** Stops following the open level, while provisioned. */
  private unfollow: Nullable<() => void> = null;

  public constructor(
    private readonly loadService: LevelLoadService = inject(LevelLoadService),
    private readonly renderSettingsService: RenderSettingsService = inject(RenderSettingsService)
  ) {
    this.choice = toLevelLookChoice(parseLocalStorageValueSafe(LEVEL_LOOK_STORAGE_KEY), this.exposure);
  }

  /** Reads each level's defaults as it opens, whether or not a viewport draws it. */
  @OnProvision()
  public onProvision(): void {
    this.unfollow = this.loadService.followLevel((selected) => void this.open(selected));
  }

  @OnDeprovision()
  public onDeprovision(): void {
    this.unfollow?.();
    this.unfollow = null;
  }

  /**
   * The look the open level's game ships, its engine's own where the game leaves a command to it; null before its
   * defaults are read or with none open.
   */
  @Computed()
  public get game(): Nullable<ILevelLook> {
    const defaults: Nullable<LevelConsoleDefaults> = this.defaults.value;

    return defaults && this.engine ? toGameLevelLook(defaults, this.exposure, this.engine) : null;
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
      this.defaults = sessionId ? AsyncState.loading() : AsyncState.idle();
      this.engine = selected?.value.engine.engine ?? null;
    });

    if (!sessionId) {
      return;
    }

    try {
      const { value } = await levelsCommands.describeConsoleDefaults(sessionId);

      if (this.sessionId === sessionId) {
        runInAction(() => {
          this.defaults = AsyncState.ready(value);
        });
      }
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.warn("The game's console defaults cannot be read, so its levels look as the settings say:", transformed);

      if (this.sessionId === sessionId) {
        runInAction(() => {
          this.defaults = AsyncState.failed(transformed);
        });
      }
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
    return this.renderSettingsService.rendererFeatures.exposure;
  }

  private store(choice: ILevelLookChoice): void {
    this.choice = choice;
    setLocalStorageValueSafe(LEVEL_LOOK_STORAGE_KEY, JSON.stringify(choice));
  }
}
