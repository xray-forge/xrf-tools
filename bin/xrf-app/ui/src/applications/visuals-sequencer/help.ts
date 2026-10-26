import { EApplicationId, IApplicationHelp } from "@/core/routing/application";

export const VISUALS_SEQUENCER_HELP: IApplicationHelp = {
  summary:
    "Preview an ordered sequence of an OGF model's motions. Add clips to a track, rearrange them, and play " +
    "them together to inspect how animations follow one another.",
  workflow: [
    "Choose a loose `.ogf` file. If its referenced assets live elsewhere, name a gamedata tree or installation " +
      "under `Also search in`, then select `Open`.",
    "Find motions in the left panel and use their add actions to build the `Sequence` track.",
    "Move clips earlier or later, remove them, or select a clip name to seek to its beginning.",
    "Use `Play`, the clip navigation buttons, and the frame slider to inspect playback. Adjust playback rate " +
      "or toggle `Loop` to repeat the track.",
  ],
  nuances: [
    "A motion is prepared when added to the track. Clips show `Baking` while that work runs and `Unavailable` " +
      "with a reason when it fails; playback passes over clips it cannot play.",
    "The same motion can appear more than once in a track.",
    "Clips play in order with a cut at each boundary. The loop control repeats the whole track.",
    "The duration totals come from the prepared motions. Changing the preview playback rate does not change " +
      "those reported durations.",
  ],
  limitations: [
    "The track is temporary: opening another model, leaving the application, or reloading loses it. There is " +
      "no sequence save or export.",
    "The sequencer previews existing motions from one model. It does not author animation frames or blend transitions.",
    "Opening accepts a loose OGF file, not a model selected inside an archive. A model without usable motions " +
      "has nothing to add to the track.",
  ],
  relatedTools: [EApplicationId.VISUALS_EXPLORER, EApplicationId.ARCHIVES_EXPLORER],
};
