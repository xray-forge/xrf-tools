import { beforeEach, describe, expect, it } from "@jest/globals";

import { EXrayEngineChoice } from "@/core/ipc/types/xrf-engine-target";
import { ENGINE_CHOICES_STORAGE_KEY } from "@/core/storage";

import { readEngineChoice, toEngineChoiceMemoryKey, writeEngineChoice } from "./engine-choice-memory";

describe("engine choice memory", () => {
  beforeEach(() => {
    window.localStorage.clear();
  });

  it("leaves a root it was never told about to detection", () => {
    expect(readEngineChoice("C:\\Games\\Anomaly")).toBe(EXrayEngineChoice.AUTO);
    expect(readEngineChoice(null)).toBe(EXrayEngineChoice.AUTO);
  });

  it("remembers an override per root, whatever case or trailing separator it is named with", () => {
    writeEngineChoice("C:\\Games\\Anomaly\\", EXrayEngineChoice.VANILLA);

    expect(readEngineChoice("c:\\games\\anomaly")).toBe(EXrayEngineChoice.VANILLA);
    expect(readEngineChoice("C:\\Games\\CoP")).toBe(EXrayEngineChoice.AUTO);
    expect(toEngineChoiceMemoryKey("C:/Games/Anomaly//")).toBe("c:/games/anomaly");
  });

  it("forgets the override once the root is left to detection again", () => {
    writeEngineChoice("C:\\Games\\Anomaly", EXrayEngineChoice.EXTENDED);
    writeEngineChoice("C:\\Games\\Anomaly", EXrayEngineChoice.AUTO);

    expect(readEngineChoice("C:\\Games\\Anomaly")).toBe(EXrayEngineChoice.AUTO);
    expect(window.localStorage.getItem(ENGINE_CHOICES_STORAGE_KEY)).toBe("{}");
  });

  it("lets the least recently told root go past its limit", () => {
    for (let index: number = 0; index <= 32; index += 1) {
      writeEngineChoice(`C:\\Games\\${index}`, EXrayEngineChoice.EXTENDED);
    }

    expect(readEngineChoice("C:\\Games\\0")).toBe(EXrayEngineChoice.AUTO);
    expect(readEngineChoice("C:\\Games\\32")).toBe(EXrayEngineChoice.EXTENDED);
  });

  it("reads a memory it cannot parse as none", () => {
    window.localStorage.setItem(ENGINE_CHOICES_STORAGE_KEY, "not json");

    expect(readEngineChoice("C:\\Games\\Anomaly")).toBe(EXrayEngineChoice.AUTO);

    window.localStorage.setItem(ENGINE_CHOICES_STORAGE_KEY, JSON.stringify({ "c:\\games\\anomaly": "anomaly" }));

    expect(readEngineChoice("C:\\Games\\Anomaly")).toBe(EXrayEngineChoice.AUTO);
  });
});
