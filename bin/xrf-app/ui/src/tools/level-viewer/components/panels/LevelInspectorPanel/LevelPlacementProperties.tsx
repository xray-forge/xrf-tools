import {
  ELevelPlacementDetails,
  LevelObjectLight,
  LevelObjectPlacement,
  LevelObjectShape,
} from "@xrf/ipc/types/xrf-app-levels";
import { assertExhaustive, Nullable } from "@xrf/types";
import { EditorPanelProperty } from "@xrf/ui/editor/panel";
import { ReactElement } from "react";

import {
  describeLevelColor,
  describeLevelShape,
  describeLevelSoundWindow,
} from "@/tools/level-viewer/lib/outliner/level-object-describe";

interface ILevelPlacementPropertiesProps {
  /** What its companion file records of it, for a placement. */
  placement: Nullable<LevelObjectPlacement>;
  /** What the bundle records of it, for a static light. */
  light: Nullable<LevelObjectLight>;
}

/** A quantity as the Inspector writes it: two decimals at most, no trailing zeros. */
function formatNumber(value: Nullable<number>): string {
  return String(Number((value ?? 0).toFixed(2)));
}

/**
 * What a placement's companion file or a static light's bundle records of it, by what it is, and the shapes a
 * placement spans; nothing for any other level object.
 */
export function LevelPlacementProperties({ placement, light }: ILevelPlacementPropertiesProps): ReactElement {
  return (
    <>
      {placement ? <LevelPlacementDetailsProperties placement={placement} /> : null}

      {placement?.shapes.length ? (
        <EditorPanelProperty
          label={"Shapes"}
          value={placement.shapes.map((shape: LevelObjectShape) => describeLevelShape(shape)).join("\n")}
          isMonospace
        />
      ) : null}

      {light ? (
        <>
          <EditorPanelProperty label={"Lit as"} value={light.isSpot ? "Point, authored as a spot" : "Point"} />
          <EditorPanelProperty label={"Colour"} value={describeLevelColor(light.color)} isMonospace />
          <EditorPanelProperty label={"Range"} value={`${formatNumber(light.range)} m`} />
        </>
      ) : null}
    </>
  );
}

function LevelPlacementDetailsProperties({ placement }: { placement: LevelObjectPlacement }): ReactElement {
  const { details } = placement;

  switch (details.kind) {
    case ELevelPlacementDetails.PARTICLES:
      return <EditorPanelProperty label={"Effect"} value={details.effect} isMonospace />;

    case ELevelPlacementDetails.SOUND:
      return (
        <>
          <EditorPanelProperty label={"Sound"} value={details.sound} isMonospace />
          <EditorPanelProperty label={"Volume"} value={formatNumber(details.volume)} />
          <EditorPanelProperty label={"Frequency"} value={formatNumber(details.frequency)} />
          <EditorPanelProperty label={"Plays"} value={describeLevelSoundWindow(details.active, "active")} />
          <EditorPanelProperty label={"Each play"} value={describeLevelSoundWindow(details.play, "play")} />
          <EditorPanelProperty label={"Pause"} value={describeLevelSoundWindow(details.pause, "pause")} />
        </>
      );

    case ELevelPlacementDetails.SOUND_ENVIRONMENT:
      return (
        <>
          <EditorPanelProperty label={"Outside"} value={details.outer} />
          <EditorPanelProperty label={"Inside"} value={details.inner} />
        </>
      );

    case ELevelPlacementDetails.FOG_VOLUME:
      return (
        <>
          <EditorPanelProperty label={"Profile"} value={details.profile} isMonospace />
          <EditorPanelProperty label={"Obstacles"} value={details.obstacles} />
        </>
      );

    case ELevelPlacementDetails.ENV_MODIFIER:
      return (
        <>
          <EditorPanelProperty label={"Power"} value={formatNumber(details.power)} />
          <EditorPanelProperty
            label={"Moves"}
            value={details.affects.length ? details.affects.join(", ") : "Nothing"}
          />
          <EditorPanelProperty label={"Far plane"} value={`${formatNumber(details.farPlane)} m`} />
          <EditorPanelProperty label={"Fog colour"} value={describeLevelColor(details.fogColor)} isMonospace />
          <EditorPanelProperty label={"Fog density"} value={formatNumber(details.fogDensity)} />
          <EditorPanelProperty label={"Ambient"} value={describeLevelColor(details.ambient)} isMonospace />
          <EditorPanelProperty label={"Sky"} value={describeLevelColor(details.skyColor)} isMonospace />
          <EditorPanelProperty label={"Hemisphere"} value={describeLevelColor(details.hemiColor)} isMonospace />
        </>
      );

    case ELevelPlacementDetails.RESPAWN_POINT:
      return (
        <>
          <EditorPanelProperty label={"Respawns"} value={details.role} />
          <EditorPanelProperty label={"Team"} value={details.team} />
          <EditorPanelProperty label={"Game types"} value={`0x${details.gameTypes.toString(16)}`} isMonospace />
          <EditorPanelProperty label={"Profile"} value={details.profile || "None"} isMonospace />
        </>
      );

    default:
      return assertExhaustive(details);
  }
}
