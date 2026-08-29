import { default as AnimationIcon } from "@mui/icons-material/Animation";
import { default as PlaylistPlayIcon } from "@mui/icons-material/PlaylistPlay";

import { SequenceMotionsPanel } from "@/applications/visuals-sequencer/components/panels/SequenceMotionsPanel";
import { SequenceTrackPanel } from "@/applications/visuals-sequencer/components/panels/SequenceTrackPanel";
import { IEditorPanel } from "@/core/shell/panel/context";
import { VISUAL_PANELS } from "@/core/visuals/components/panels/visual-panels";

/**
 * What the sequencer contributes to the panel stripes.
 *
 * Both open by default, because neither is an inspection detail: the motions are the material and the track is the
 * document, and a sequencer showing neither is a viewer.
 */
export const SEQUENCER_PANELS: Array<IEditorPanel> = [
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
