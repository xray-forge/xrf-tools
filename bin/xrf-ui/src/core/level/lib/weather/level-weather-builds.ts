import { IRendererWeather } from "@xrf/renderer";
import { Nullable } from "@xrf/types";

import { levelsCommands } from "@/core/ipc/commands/levels";
import {
  LevelTextureReference,
  LevelWeatherDescription,
  SelectedLevelDescription,
  SessionSnapshot,
} from "@/core/ipc/types/xrf-app";
import { XrayEngine } from "@/core/ipc/types/xrf-engine-target";
import { listLevelManualWeatherTextures } from "@/core/level/lib/weather/level-manual-weather";
import { LevelManualWeatherBuilder } from "@/core/level/lib/weather/level-manual-weather-builder";
import { toLevelRendererWeather, toLevelRendererWeatherBase } from "@/core/level/lib/weather/level-renderer-weather";
import { TLevelRendererWeatherBase } from "@/core/level/lib/weather/level-renderer-weather-base";
import { ILevelWeatherCycleBuildInput } from "@/core/level/lib/weather/level-weather-cycle-build-input";
import { ILevelWeatherManualBuildInput } from "@/core/level/lib/weather/level-weather-manual-build-input";

/**
 * What one open level's weathers are built into for the renderer: the parts every weather of the level plays with,
 * built once, the keyframe set by hand over them, and where each texture that names resolves in the level. One a
 * level, so nothing of one level's is ever built into another's.
 */
export class LevelWeatherBuilds {
  private readonly selected: SessionSnapshot<SelectedLevelDescription>;
  /** What each texture the keyframe set by hand named came to, so an edit that names none new asks for nothing. */
  private readonly located: Map<string, LevelTextureReference> = new Map();
  /** What every weather of the level plays with, for the description it was built from. */
  private base: Nullable<{ description: LevelWeatherDescription; built: Promise<TLevelRendererWeatherBase> }> = null;
  /** What builds the keyframe set by hand, for the description it plays over. */
  private manual: Nullable<{ description: Nullable<LevelWeatherDescription>; builder: LevelManualWeatherBuilder }> =
    null;

  /**
   * @param selected - The level open, which its textures are found and read in.
   */
  public constructor(selected: SessionSnapshot<SelectedLevelDescription>) {
    this.selected = selected;
  }

  /**
   * @param input - The cycle and the level's weather it plays over.
   * @returns What the renderer plays of the cycle.
   */
  public async buildCycle(input: ILevelWeatherCycleBuildInput): Promise<IRendererWeather> {
    return toLevelRendererWeather({
      base: await this.toBase(input.description),
      cycle: input.cycle,
      roots: this.selected.value.roots,
    });
  }

  /**
   * @param input - The keyframe, the level's weather it plays over, and the engine target.
   * @returns What the renderer plays of the keyframe, its textures resolved as the level resolves its own.
   */
  public async buildManual(input: ILevelWeatherManualBuildInput): Promise<IRendererWeather> {
    const references: Array<string> = listLevelManualWeatherTextures(input.manual);

    await this.locate(references);

    const builder: LevelManualWeatherBuilder = await this.toManualBuilder(input.description, input.engine);

    return builder.build(
      input.manual,
      references.flatMap((reference: string) => this.located.get(reference) ?? [])
    );
  }

  /** Asks where the textures not asked about before resolve. */
  private async locate(references: ReadonlyArray<string>): Promise<void> {
    const unknown: Array<string> = references.filter((reference: string) => !this.located.has(reference));

    if (unknown.length) {
      const { value } = await levelsCommands.resolveLevelTextures(this.selected.sessionId, unknown);

      value.forEach((it: LevelTextureReference) => this.located.set(it.reference, it));
    }
  }

  /** What every weather of the level plays with, built the first time it is asked for; one that failed is built again. */
  private toBase(description: LevelWeatherDescription): Promise<TLevelRendererWeatherBase> {
    if (this.base?.description !== description) {
      const built: Promise<TLevelRendererWeatherBase> = toLevelRendererWeatherBase({
        description,
        roots: this.selected.value.roots,
      });

      this.base = { built, description };
      built.catch(() => {
        if (this.base?.built === built) {
          this.base = null;
        }
      });
    }

    return this.base.built;
  }

  /** What builds the keyframe set by hand over the level's weather as it reads now. */
  private async toManualBuilder(
    description: Nullable<LevelWeatherDescription>,
    engine: XrayEngine
  ): Promise<LevelManualWeatherBuilder> {
    if (!this.manual || this.manual.description !== description) {
      const builder: LevelManualWeatherBuilder = new LevelManualWeatherBuilder({
        base: description ? await this.toBase(description) : null,
        engine,
        roots: this.selected.value.roots,
      });

      this.manual = { builder, description };
    }

    return this.manual.builder;
  }
}
