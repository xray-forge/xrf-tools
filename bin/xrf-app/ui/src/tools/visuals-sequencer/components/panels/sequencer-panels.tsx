import { AnimationIcon } from "@xrf/ui/icons/animation";
import { PlaylistPlayIcon } from "@xrf/ui/icons/playlist-play";

import { IShellPanel } from "@/core/shell/panels/shell-panel";
import { VISUAL_PANELS } from "@/core/visuals/components/panels/visual-panels";
import { SequenceMotionsPanel } from "@/tools/visuals-sequencer/components/panels/SequenceMotionsPanel";
import { SequenceTrackPanel } from "@/tools/visuals-sequencer/components/panels/SequenceTrackPanel";

/**
 * What the sequencer contributes to the panel stripes.
 *
 * Both open by default, because neither is an inspection detail: the motions are the material and the track is the
 * document, and a sequencer showing neither is a viewer.
 */
export const SEQUENCER_PANELS: Array<IShellPanel> = [
  {
    icon: <AnimationIcon />,
    id: "motions",
    isOpenByDefault: true,
    label: "Motions",
    render: () => <SequenceMotionsPanel />,
    side: "left",
  },
  {
    icon: <PlaylistPlayIcon />,
    id: "track",
    isOpenByDefault: true,
    label: "Sequence",
    render: () => <SequenceTrackPanel />,
  },
  ...VISUAL_PANELS,
];
