import { describe, expect, it } from "@jest/globals";
import { EWorldSoundCategory, EWorldSoundKind, WorldSoundVoiceReport } from "@xrf/ipc/types/xrf-world";

import {
  formatLevelSoundAcoustics,
  formatLevelSoundAcousticsState,
  formatLevelSoundGain,
  formatLevelSoundSeconds,
  formatLevelSoundTimeOfDay,
  formatLevelSoundTrackTime,
} from "@/tools/level-viewer/lib/sound/level-sound-format";

function voice(gain: number | null, isLoading: boolean = false): WorldSoundVoiceReport {
  return {
    category: EWorldSoundCategory.AMBIENCE,
    distance: 10,
    gain,
    occlusion: null,
    transmission: null,
    reverbSend: null,
    isBinaural: false,
    id: 1,
    is2d: false,
    isLoading,
    isReal: true,
    kind: EWorldSoundKind.STATIC,
    object: null,
    position: [0, 0, 0],
    restarts: 0,
    sound: "ambient\\wind",
  };
}

describe("level sound format", () => {
  it("says a gain in decibels, a culled one silent and a sound being read as such", () => {
    expect(formatLevelSoundGain(voice(0.5))).toBe("-6.0 dB");
    expect(formatLevelSoundGain(voice(0))).toBe("Silent");
    expect(formatLevelSoundGain(voice(null))).toBe("Silent");
    expect(formatLevelSoundGain(voice(1, true))).toBe("Reading");
  });

  it("says how the walls and the reverb shape a placed sound, and nothing before the audio took it", () => {
    expect(formatLevelSoundAcoustics(voice(0.5))).toBe("");
    expect(
      formatLevelSoundAcoustics({
        ...voice(0.5),
        occlusion: 0.25,
        transmission: [0.1, 0.05, 0.02],
        reverbSend: 0.5,
        isBinaural: true,
      })
    ).toBe("25% unobstructed, walls pass 10%/5%/2%, reverb 0.50, HRTF");
    expect(formatLevelSoundAcoustics({ ...voice(0.5), occlusion: 1, transmission: [1, 1, 1], reverbSend: 0 })).toBe(
      "unobstructed, panned"
    );
  });

  it("says whether the acoustics run, on how many triangles, or why the scene plays dry", () => {
    const running = {
      failure: null,
      triangles: 1200,
      geometryFailure: null,
      reverbTimes: [1.5, 1.25, 0.5] as [number, number, number],
      buildMs: 900,
      directMs: 2,
      reflectionsMs: 30,
    };

    expect(formatLevelSoundAcousticsState(null)).toBe("Not yet heard");
    expect(formatLevelSoundAcousticsState(running)).toBe("Steam Audio, 1,200 triangles, reverb 1.50/1.25/0.50 s");
    expect(formatLevelSoundAcousticsState({ ...running, triangles: null })).toBe(
      "Steam Audio, building the level's walls"
    );
    expect(formatLevelSoundAcousticsState({ ...running, failure: "No library" })).toBe("Dry: No library");
  });

  it("says the level's time of day and waits in whole seconds", () => {
    expect(formatLevelSoundTimeOfDay(23 * 3_600_000 + 5 * 60_000)).toBe("23:05");
    expect(formatLevelSoundSeconds(1200)).toBe("2 s");
  });

  it("says how far into a track in minutes and seconds", () => {
    expect(formatLevelSoundTrackTime(0)).toBe("0:00");
    expect(formatLevelSoundTrackTime(65_900)).toBe("1:05");
    expect(formatLevelSoundTrackTime(-10)).toBe("0:00");
  });
});
