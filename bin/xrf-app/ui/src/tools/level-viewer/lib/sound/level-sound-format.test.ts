import { describe, expect, it } from "@jest/globals";
import { EWorldSoundCategory, EWorldSoundKind, WorldSoundVoiceReport } from "@xrf/ipc/types/xrf-world";

import {
  formatLevelSoundGain,
  formatLevelSoundSeconds,
  formatLevelSoundTimeOfDay,
} from "@/tools/level-viewer/lib/sound/level-sound-format";

function voice(gain: number | null, isLoading: boolean = false): WorldSoundVoiceReport {
  return {
    category: EWorldSoundCategory.AMBIENCE,
    distance: 10,
    gain,
    id: 1,
    is2d: false,
    isLoading,
    isReal: true,
    kind: EWorldSoundKind.STATIC,
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

  it("says the level's time of day and waits in whole seconds", () => {
    expect(formatLevelSoundTimeOfDay(23 * 3_600_000 + 5 * 60_000)).toBe("23:05");
    expect(formatLevelSoundSeconds(1200)).toBe("2 s");
  });
});
