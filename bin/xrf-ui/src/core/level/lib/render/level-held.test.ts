import { describe, expect, it } from "@jest/globals";
import { Nullable } from "@xrf/types";

import { LevelHeld } from "@/core/level/lib/render/level-held";

describe("LevelHeld", () => {
  it("tells a listener what is held at once, then each value held", () => {
    const held: LevelHeld<string> = new LevelHeld();
    const told: Array<Nullable<string>> = [];

    held.hold("grass");
    held.subscribe((value: Nullable<string>) => told.push(value));
    held.hold("lights");

    expect(told).toEqual(["grass", "lights"]);
  });

  it("keeps the textures claimed ahead of the value, telling nobody until it is held", () => {
    const held: LevelHeld<string> = new LevelHeld();
    const told: Array<Nullable<string>> = [];

    held.subscribe((value: Nullable<string>) => told.push(value));
    held.claim(new Set(["lamp"]));

    expect(told).toEqual([null]);
    expect(Array.from(held.textures)).toEqual(["lamp"]);

    held.hold("lights");

    expect(told).toEqual([null, "lights"]);
    expect(Array.from(held.textures)).toEqual(["lamp"]);
  });

  it("lets the textures go with the value, and tells nobody where nothing was held", () => {
    const held: LevelHeld<string> = new LevelHeld();
    const told: Array<Nullable<string>> = [];

    held.subscribe((value: Nullable<string>) => told.push(value));
    held.claim(new Set(["lamp"]));
    held.release();

    expect(told).toEqual([null]);
    expect(held.textures.size).toBe(0);

    held.hold("lights");
    held.release();

    expect(told).toEqual([null, "lights", null]);
  });
});
