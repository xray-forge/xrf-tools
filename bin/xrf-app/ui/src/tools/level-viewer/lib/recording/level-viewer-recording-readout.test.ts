import { describe, expect, it } from "@jest/globals";
import { RecordingOutcome } from "@xrf/ipc/types/xrf-app-levels";

import { LevelViewerRecordingReadout } from "@/tools/level-viewer/lib/recording/level-viewer-recording-readout";

const OUTCOME: RecordingOutcome = {
  file: "C:\\appdata\\level-viewer\\recordings\\zaton-20261104-232837.mp4",
  duration: 15.1,
  frames: 906,
  droppedFrames: 0,
  droppedSound: 0,
  width: 3280,
  height: 1930,
  encoder: "NVIDIA H.264 Encoder MFT (hardware)",
};

describe("LevelViewerRecordingReadout", () => {
  it("tells a recording running, before and after its first word", () => {
    expect(LevelViewerRecordingReadout.describeProgress(null)).toBe("Recording…");
    expect(
      LevelViewerRecordingReadout.describeProgress({
        elapsed: 2.1,
        frames: 126,
        droppedFrames: 2,
        droppedSound: 1,
        ended: null,
      })
    ).toMatch(/^Recording .+, 2 frames and 1 sound blocks dropped$/);
  });

  it("tells a recording written, by its file's name, its size and what it dropped", () => {
    expect(LevelViewerRecordingReadout.getFileName(OUTCOME.file)).toBe("zaton-20261104-232837.mp4");
    expect(LevelViewerRecordingReadout.describeOutcome(OUTCOME)).toMatch(/ · 3280 × 1930 · nothing dropped$/);
    expect(LevelViewerRecordingReadout.hasDrops(OUTCOME)).toBe(false);
    expect(LevelViewerRecordingReadout.hasDrops({ ...OUTCOME, droppedSound: 1 })).toBe(true);
  });
});
