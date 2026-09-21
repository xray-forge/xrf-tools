import { describe, expect, it } from "@jest/globals";

import {
  EMPTY_LEVEL_STREAM_SUMMARY,
  ILevelStreamReading,
  ILevelStreamSummary,
  LevelStreamProfile,
  STREAM_WINDOW,
} from "@/core/level/lib/stream/level-stream-profile";

function mockReading(sector: number, total: number, stages: Partial<ILevelStreamReading> = {}): ILevelStreamReading {
  return {
    adopt: 0,
    draws: 1,
    pack: 0,
    sector,
    textures: 0,
    total,
    transfer: 0,
    vertices: 1,
    views: 0,
    ...stages,
  };
}

describe("LevelStreamProfile", () => {
  it("reports nothing before a sector has been read", () => {
    expect(new LevelStreamProfile().summarise()).toEqual(EMPTY_LEVEL_STREAM_SUMMARY);
  });

  // The point of the record: a stutter is one stage, and a total alone never says which.
  it("means each stage separately", () => {
    const profile: LevelStreamProfile = new LevelStreamProfile();

    profile.record(mockReading(1, 100, { pack: 80, textures: 20 }));
    profile.record(mockReading(2, 40, { pack: 20, textures: 20 }));

    const summary: ILevelStreamSummary = profile.summarise();

    expect(summary.mean?.pack).toBe(50);
    expect(summary.mean?.textures).toBe(20);
    expect(summary.mean?.total).toBe(70);
  });

  it("keeps the last read and the worst of them apart", () => {
    const profile: LevelStreamProfile = new LevelStreamProfile();

    profile.record(mockReading(1, 200));
    profile.record(mockReading(2, 10));

    const summary: ILevelStreamSummary = profile.summarise();

    expect(summary.last?.sector).toBe(2);
    expect(summary.worst?.sector).toBe(1);
  });

  // The mean follows the flight rather than the session: a level opened an hour ago should not still be averaging
  // the sectors it read at the door.
  it("means only the window, while counting every read", () => {
    const profile: LevelStreamProfile = new LevelStreamProfile();

    profile.record(mockReading(0, 1000));

    for (let at = 1; at <= STREAM_WINDOW; at += 1) {
      profile.record(mockReading(at, 10));
    }

    const summary: ILevelStreamSummary = profile.summarise();

    expect(summary.sectors).toBe(STREAM_WINDOW + 1);
    expect(summary.mean?.total).toBe(10);
    expect(summary.worst?.total).toBe(10);
  });

  it("forgets everything when a level goes", () => {
    const profile: LevelStreamProfile = new LevelStreamProfile();

    profile.record(mockReading(1, 100));
    profile.clear();

    expect(profile.summarise()).toEqual(EMPTY_LEVEL_STREAM_SUMMARY);
  });
});
