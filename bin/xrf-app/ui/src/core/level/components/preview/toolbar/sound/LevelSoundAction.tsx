import { useInjection } from "@wirestate/react";
import { EditorPopoverGroup, EditorPopoverGroupSection } from "@xrf/ui/editor/popover-group";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { VolumeUpIcon } from "@xrf/ui/icons/volume-up";
import { ReactElement } from "react";

import { LevelViewService } from "@/core/level/services/level-view";

/**
 * Whether the level's sound is heard while its viewport is the one listening, and which of its categories: a sound
 * switched off is silent and its schedule runs on, so switching it back on hears it where it would be.
 */
export function LevelSoundAction({
  "data-testid": dataTestId = "level-sound-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const viewService: LevelViewService = useInjection(LevelViewService);

  const { isSoundHeard, isMusicHeard, isAmbienceHeard, isWorldHeard } = viewService.options;

  return (
    <EditorPopoverGroup
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Sound"}
      description={isSoundHeard ? "Sound heard while this viewport listens" : "Sound off"}
      icon={<VolumeUpIcon />}
      isActive={isSoundHeard}
    >
      <EditorPopoverGroupSection
        label={"Sound"}
        description={"Plays the level as the game does, from the camera of the focused viewport"}
        isOn={isSoundHeard}
        onToggle={() => viewService.toggleOption("isSoundHeard")}
      />

      <EditorPopoverGroupSection
        label={"Music"}
        description={"The level's music tracks, picked by the hour"}
        isOn={isMusicHeard}
        onToggle={() => viewService.toggleOption("isMusicHeard")}
      />

      <EditorPopoverGroupSection
        label={"Ambience"}
        description={"Static sounds, the weather's channels, rain, thunder and ambient effects"}
        isOn={isAmbienceHeard}
        onToggle={() => viewService.toggleOption("isAmbienceHeard")}
      />

      <EditorPopoverGroupSection
        label={"World"}
        description={"Sounds at the world's objects: anomalies and campfires"}
        isOn={isWorldHeard}
        onToggle={() => viewService.toggleOption("isWorldHeard")}
      />
    </EditorPopoverGroup>
  );
}
