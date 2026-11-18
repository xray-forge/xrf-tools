import { describe, expect, it } from "@jest/globals";
import {
  EAiMapCovers,
  ELevelCompileStep,
  ELightQuality,
  ERayBackend,
  LevelCompileStep,
} from "@xrf/ipc/types/xrf-level-compiler";

import { ELevelCompilerField } from "@/core/level/lib/compiler/level-compiler-field";

import { getCompileProblem, ICompileChoice, toCompileRequest, toggleCompileStep } from "./compile-request";

function choice(steps: Array<LevelCompileStep>, patch: Partial<ICompileChoice> = {}): ICompileChoice {
  return {
    steps: new Set(steps),
    gameMaterials: "C:\\Games\\CoP\\gamedata\\gamemtl.xr",
    gamedata: "C:\\Games\\CoP\\gamedata",
    textures: "C:\\Games\\CoP\\gamedata\\textures",
    rayBackend: ERayBackend.AUTO,
    rayGpu: { vendor: 0x10de, device: 0x2c02 },
    aiCovers: EAiMapCovers.FULL,
    lightQuality: ELightQuality.SCENE,
    ...patch,
  };
}

describe("toCompileRequest", () => {
  it("builds alone where no step is chosen, whatever else is named", () => {
    expect(toCompileRequest(choice([]))).toBeNull();
    expect(getCompileProblem(choice([], { gamedata: null }))).toBeNull();
  });

  it("asks for the steps in the order the compile runs them", () => {
    const request = toCompileRequest(
      choice([ELevelCompileStep.AI, ELevelCompileStep.LIGHT, ELevelCompileStep.GEOMETRY])
    );

    expect(request?.steps).toEqual([ELevelCompileStep.GEOMETRY, ELevelCompileStep.LIGHT, ELevelCompileStep.AI]);
    expect(request?.textures).toEqual(["C:\\Games\\CoP\\gamedata\\textures"]);
  });

  it("asks for the rays traced as chosen, on the renderer's GPU, the AI map's covers and the light's quality", () => {
    const request = toCompileRequest(
      choice([ELevelCompileStep.GEOMETRY, ELevelCompileStep.LIGHT, ELevelCompileStep.AI], {
        rayBackend: ERayBackend.CPU,
        aiCovers: EAiMapCovers.NONE,
        lightQuality: ELightQuality.DRAFT,
      })
    );

    expect(request?.backend).toBe(ERayBackend.CPU);
    expect(request?.gpu).toEqual({ vendor: 0x10de, device: 0x2c02 });
    expect(request?.aiCovers).toBe(EAiMapCovers.NONE);
    expect(request?.lightQuality).toBe(ELightQuality.DRAFT);
  });

  it("names what a chosen step lacks before it can run", () => {
    expect(getCompileProblem(choice([ELevelCompileStep.LIGHT]))).toEqual({
      field: null,
      message: expect.stringContaining("geometry"),
    });
    expect(toCompileRequest(choice([ELevelCompileStep.LIGHT, ELevelCompileStep.DETAILS]))).toBeNull();
    expect(getCompileProblem(choice([ELevelCompileStep.DETAILS], { gamedata: null }))?.field).toBe(
      ELevelCompilerField.GAMEDATA
    );
    expect(toCompileRequest(choice([ELevelCompileStep.DETAILS], { gamedata: null }))).toBeNull();
    expect(getCompileProblem(choice([ELevelCompileStep.AI], { gameMaterials: null }))?.field).toBe(
      ELevelCompilerField.GAME_MATERIALS
    );
    expect(toCompileRequest(choice([ELevelCompileStep.AI], { gameMaterials: null }))).toBeNull();
    expect(getCompileProblem(choice([], { gameMaterials: null }))).toBeNull();
  });

  it("names no textures root where none is chosen, leaving the game data's to the backend", () => {
    expect(getCompileProblem(choice([ELevelCompileStep.DETAILS], { textures: null }))).toBeNull();
    expect(toCompileRequest(choice([ELevelCompileStep.DETAILS], { textures: null }))?.textures).toEqual([]);
  });
});

describe("toggleCompileStep", () => {
  it("ticks the light with the geometry and drops it with the geometry", () => {
    const ticked = toggleCompileStep(new Set(), ELevelCompileStep.GEOMETRY, true);

    expect([...ticked]).toEqual([ELevelCompileStep.GEOMETRY, ELevelCompileStep.LIGHT]);
    expect([...toggleCompileStep(ticked, ELevelCompileStep.GEOMETRY, false)]).toEqual([]);
  });

  it("lets the light be unticked alone, for geometry under neutral light", () => {
    const ticked = toggleCompileStep(new Set(), ELevelCompileStep.GEOMETRY, true);

    expect([...toggleCompileStep(ticked, ELevelCompileStep.LIGHT, false)]).toEqual([ELevelCompileStep.GEOMETRY]);
  });

  it("never holds the light without the geometry", () => {
    expect([...toggleCompileStep(new Set(), ELevelCompileStep.LIGHT, true)]).toEqual([]);
  });
});
