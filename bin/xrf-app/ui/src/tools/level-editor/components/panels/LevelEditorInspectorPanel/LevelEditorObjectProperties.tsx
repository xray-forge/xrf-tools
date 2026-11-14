import { ESceneObjectProperties, SceneObjectProperties } from "@xrf/ipc/types/xrf-app-levels";
import { assertExhaustive } from "@xrf/types";
import { ReactElement } from "react";

import { LevelEditorFogVolumeProperties } from "./LevelEditorFogVolumeProperties";
import { LevelEditorGlowProperties } from "./LevelEditorGlowProperties";
import { LevelEditorGroupProperties } from "./LevelEditorGroupProperties";
import { LevelEditorLightProperties } from "./LevelEditorLightProperties";
import { LevelEditorParticleProperties } from "./LevelEditorParticleProperties";
import { LevelEditorPortalProperties } from "./LevelEditorPortalProperties";
import { LevelEditorSceneObjectProperties } from "./LevelEditorSceneObjectProperties";
import { LevelEditorSectorProperties } from "./LevelEditorSectorProperties";
import { LevelEditorShapeProperties } from "./LevelEditorShapeProperties";
import { LevelEditorSoundEnvironmentProperties } from "./LevelEditorSoundEnvironmentProperties";
import { LevelEditorSoundSourceProperties } from "./LevelEditorSoundSourceProperties";
import { LevelEditorSpawnProperties } from "./LevelEditorSpawnProperties";
import { LevelEditorWayProperties } from "./LevelEditorWayProperties";

interface ILevelEditorObjectPropertiesProps {
  properties: SceneObjectProperties;
}

/**
 * What one scene object's kind keeps beyond its name and transform, by the tool it belongs to.
 */
export function LevelEditorObjectProperties({ properties }: ILevelEditorObjectPropertiesProps): ReactElement {
  switch (properties.kind) {
    case ESceneObjectProperties.SCENE_OBJECT:
      return <LevelEditorSceneObjectProperties properties={properties} />;

    case ESceneObjectProperties.SECTOR:
      return <LevelEditorSectorProperties properties={properties} />;

    case ESceneObjectProperties.PORTAL:
      return <LevelEditorPortalProperties properties={properties} />;

    case ESceneObjectProperties.LIGHT:
      return <LevelEditorLightProperties properties={properties} />;

    case ESceneObjectProperties.GLOW:
      return <LevelEditorGlowProperties properties={properties} />;

    case ESceneObjectProperties.SPAWN:
      return <LevelEditorSpawnProperties properties={properties} />;

    case ESceneObjectProperties.SHAPE:
      return <LevelEditorShapeProperties properties={properties} />;

    case ESceneObjectProperties.WAY:
      return <LevelEditorWayProperties properties={properties} />;

    case ESceneObjectProperties.PARTICLE:
      return <LevelEditorParticleProperties properties={properties} />;

    case ESceneObjectProperties.SOUND_SOURCE:
      return <LevelEditorSoundSourceProperties properties={properties} />;

    case ESceneObjectProperties.SOUND_ENVIRONMENT:
      return <LevelEditorSoundEnvironmentProperties properties={properties} />;

    case ESceneObjectProperties.FOG_VOLUME:
      return <LevelEditorFogVolumeProperties properties={properties} />;

    case ESceneObjectProperties.GROUP:
      return <LevelEditorGroupProperties properties={properties} />;

    default:
      return assertExhaustive(properties);
  }
}
