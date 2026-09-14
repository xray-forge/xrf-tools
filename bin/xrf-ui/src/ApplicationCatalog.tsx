import { default as ArchiveIcon } from "@mui/icons-material/Archive";
import { default as BlurOnIcon } from "@mui/icons-material/BlurOn";
import { default as CodeIcon } from "@mui/icons-material/Code";
import { default as FactCheckIcon } from "@mui/icons-material/FactCheck";
import { default as ForumIcon } from "@mui/icons-material/Forum";
import { default as GradientIcon } from "@mui/icons-material/Gradient";
import { default as GraphicEqIcon } from "@mui/icons-material/GraphicEq";
import { default as ImageIcon } from "@mui/icons-material/Image";
import { default as LayersIcon } from "@mui/icons-material/Layers";
import { default as MapIcon } from "@mui/icons-material/Map";
import { default as SettingsApplicationsIcon } from "@mui/icons-material/SettingsApplications";
import { default as SportsEsportsIcon } from "@mui/icons-material/SportsEsports";
import { default as TerrainIcon } from "@mui/icons-material/Terrain";
import { default as TextureIcon } from "@mui/icons-material/Texture";
import { default as TranslateIcon } from "@mui/icons-material/Translate";
import { default as ViewInArIcon } from "@mui/icons-material/ViewInAr";
import { default as WbSunnyIcon } from "@mui/icons-material/WbSunny";

import { ARCHIVES_EXPLORER_APPLICATION } from "@/applications/archives-explorer";
import { ARCHIVES_PACKER_APPLICATION } from "@/applications/archives-packer";
import { ARCHIVES_PATCHER_APPLICATION } from "@/applications/archives-patcher";
import { ARCHIVES_UNPACKER_APPLICATION } from "@/applications/archives-unpacker";
import { CHARACTERS_EXPLORER_APPLICATION } from "@/applications/characters-explorer";
import { CONFIGS_EXPLORER_APPLICATION } from "@/applications/configs-explorer";
import { CONFIGS_FORMATTER_APPLICATION } from "@/applications/configs-formatter";
import { CONFIGS_VERIFIER_APPLICATION } from "@/applications/configs-verifier";
import { DIALOGS_EDITOR_APPLICATION } from "@/applications/dialogs-editor";
import { EXPORTS_EXPLORER_APPLICATION } from "@/applications/exports-explorer";
import { GAME_MATERIALS_EDITOR_APPLICATION } from "@/applications/game-materials-editor";
import { GAMEDATA_COMPARER_APPLICATION } from "@/applications/gamedata-comparer";
import { GAMEDATA_VERIFIER_APPLICATION } from "@/applications/gamedata-verifier";
import { INFO_PORTIONS_EXPLORER_APPLICATION } from "@/applications/info-portions-explorer";
import { LEVEL_AI_COMPILER_APPLICATION } from "@/applications/level-ai-compiler";
import { LEVEL_COMPILER_APPLICATION } from "@/applications/level-compiler";
import { LEVEL_DECOMPILER_APPLICATION } from "@/applications/level-decompiler";
import { LEVEL_DETAILS_COMPILER_APPLICATION } from "@/applications/level-details-compiler";
import { LEVEL_EDITOR_APPLICATION } from "@/applications/level-editor";
import { LEVEL_VIEWER_APPLICATION } from "@/applications/level-viewer";
import { LIGHT_ANIMATIONS_EDITOR_APPLICATION } from "@/applications/light-animations-editor";
import { LUA_EXPORTS_EXPLORER_APPLICATION } from "@/applications/lua-exports-explorer";
import { LUA_FORMATTER_APPLICATION } from "@/applications/lua-formatter";
import { LUA_VERIFIER_APPLICATION } from "@/applications/lua-verifier";
import { MINIMAP_EDITOR_APPLICATION } from "@/applications/minimap-editor";
import { OBJECT_LIBRARY_EDITOR_APPLICATION } from "@/applications/object-library-editor";
import { PARTICLES_EDITOR_APPLICATION } from "@/applications/particles-editor";
import { PARTICLES_EXPLORER_APPLICATION } from "@/applications/particles-explorer";
import { POSTPROCESS_EDITOR_APPLICATION } from "@/applications/postprocess-editor";
import { SHADERS_EDITOR_APPLICATION } from "@/applications/shaders-editor";
import { SHADERS_EXPLORER_APPLICATION } from "@/applications/shaders-explorer";
import { SOUND_ENVIRONMENTS_EDITOR_APPLICATION } from "@/applications/sound-environments-editor";
import { SOUNDS_EDITOR_APPLICATION } from "@/applications/sounds-editor";
import { SOUNDS_EXPLORER_APPLICATION } from "@/applications/sounds-explorer";
import { SPAWN_COMPILER_APPLICATION } from "@/applications/spawn-compiler";
import { SPAWN_EDITOR_APPLICATION } from "@/applications/spawn-editor";
import { SPAWN_PACKER_APPLICATION } from "@/applications/spawn-packer";
import { SPAWN_UNPACKER_APPLICATION } from "@/applications/spawn-unpacker";
import { SPRITE_DESCRIPTION_EDITOR_APPLICATION } from "@/applications/sprite-description-editor";
import { SPRITE_DESCRIPTION_PACKER_APPLICATION } from "@/applications/sprite-description-packer";
import { SPRITE_DESCRIPTION_UNPACKER_APPLICATION } from "@/applications/sprite-description-unpacker";
import { SPRITE_EQUIPMENT_EDITOR_APPLICATION } from "@/applications/sprite-equipment-editor";
import { SPRITE_EQUIPMENT_PACKER_APPLICATION } from "@/applications/sprite-equipment-packer";
import { SPRITE_EQUIPMENT_UNPACKER_APPLICATION } from "@/applications/sprite-equipment-unpacker";
import { TASKS_EXPLORER_APPLICATION } from "@/applications/tasks-explorer";
import { TEXTURES_EDITOR_APPLICATION } from "@/applications/textures-editor";
import { TEXTURES_EXPLORER_APPLICATION } from "@/applications/textures-explorer";
import { TRANSLATIONS_BUILDER_APPLICATION } from "@/applications/translations-builder";
import { TRANSLATIONS_EDITOR_APPLICATION } from "@/applications/translations-editor";
import { TRANSLATIONS_FORMATTER_APPLICATION } from "@/applications/translations-formatter";
import { TRANSLATIONS_PARSER_APPLICATION } from "@/applications/translations-parser";
import { TRANSLATIONS_VERIFIER_APPLICATION } from "@/applications/translations-verifier";
import { VISUALS_CONVERTER_APPLICATION } from "@/applications/visuals-converter";
import { VISUALS_EDITOR_APPLICATION } from "@/applications/visuals-editor";
import { VISUALS_EXPLORER_APPLICATION } from "@/applications/visuals-explorer";
import { VISUALS_SEQUENCER_APPLICATION } from "@/applications/visuals-sequencer";
import { WEATHER_EDITOR_APPLICATION } from "@/applications/weather-editor";
import { EApplicationGroupId, IApplicationDescriptor, IApplicationGroup } from "@/core/routing/application";
import { Nullable } from "@/lib/types/general";

export interface IApplicationCatalog {
  readonly applications: ReadonlyArray<IApplicationDescriptor>;
  readonly groups: ReadonlyArray<IApplicationGroup>;
  findApplicationByPath(pathname: string): Nullable<IApplicationDescriptor>;
  findApplicationById(id: string): Nullable<IApplicationDescriptor>;
  findApplicationGroupById(id: string): Nullable<IApplicationGroup>;
}

export const APPLICATION_CATALOG: IApplicationCatalog = {
  applications: [
    ARCHIVES_EXPLORER_APPLICATION,
    ARCHIVES_PACKER_APPLICATION,
    ARCHIVES_PATCHER_APPLICATION,
    ARCHIVES_UNPACKER_APPLICATION,
    CONFIGS_EXPLORER_APPLICATION,
    CONFIGS_VERIFIER_APPLICATION,
    CONFIGS_FORMATTER_APPLICATION,
    DIALOGS_EDITOR_APPLICATION,
    WEATHER_EDITOR_APPLICATION,
    LIGHT_ANIMATIONS_EDITOR_APPLICATION,
    POSTPROCESS_EDITOR_APPLICATION,
    GAMEDATA_VERIFIER_APPLICATION,
    GAMEDATA_COMPARER_APPLICATION,
    CHARACTERS_EXPLORER_APPLICATION,
    INFO_PORTIONS_EXPLORER_APPLICATION,
    TASKS_EXPLORER_APPLICATION,
    LEVEL_EDITOR_APPLICATION,
    LEVEL_COMPILER_APPLICATION,
    LEVEL_DECOMPILER_APPLICATION,
    LEVEL_VIEWER_APPLICATION,
    MINIMAP_EDITOR_APPLICATION,
    LEVEL_AI_COMPILER_APPLICATION,
    LEVEL_DETAILS_COMPILER_APPLICATION,
    GAME_MATERIALS_EDITOR_APPLICATION,
    PARTICLES_EXPLORER_APPLICATION,
    PARTICLES_EDITOR_APPLICATION,
    EXPORTS_EXPLORER_APPLICATION,
    LUA_EXPORTS_EXPLORER_APPLICATION,
    LUA_VERIFIER_APPLICATION,
    LUA_FORMATTER_APPLICATION,
    SHADERS_EXPLORER_APPLICATION,
    SHADERS_EDITOR_APPLICATION,
    SOUNDS_EXPLORER_APPLICATION,
    SOUNDS_EDITOR_APPLICATION,
    SOUND_ENVIRONMENTS_EDITOR_APPLICATION,
    SPAWN_EDITOR_APPLICATION,
    SPAWN_PACKER_APPLICATION,
    SPAWN_UNPACKER_APPLICATION,
    SPAWN_COMPILER_APPLICATION,
    SPRITE_EQUIPMENT_EDITOR_APPLICATION,
    SPRITE_EQUIPMENT_PACKER_APPLICATION,
    SPRITE_EQUIPMENT_UNPACKER_APPLICATION,
    SPRITE_DESCRIPTION_EDITOR_APPLICATION,
    SPRITE_DESCRIPTION_PACKER_APPLICATION,
    SPRITE_DESCRIPTION_UNPACKER_APPLICATION,
    TEXTURES_EXPLORER_APPLICATION,
    TEXTURES_EDITOR_APPLICATION,
    TRANSLATIONS_EDITOR_APPLICATION,
    TRANSLATIONS_PARSER_APPLICATION,
    TRANSLATIONS_BUILDER_APPLICATION,
    TRANSLATIONS_VERIFIER_APPLICATION,
    TRANSLATIONS_FORMATTER_APPLICATION,
    VISUALS_EXPLORER_APPLICATION,
    VISUALS_SEQUENCER_APPLICATION,
    VISUALS_CONVERTER_APPLICATION,
    VISUALS_EDITOR_APPLICATION,
    OBJECT_LIBRARY_EDITOR_APPLICATION,
  ],
  groups: [
    {
      accent: { light: "#1c982d", dark: "#5d9f4b" },
      id: EApplicationGroupId.ARCHIVES,
      label: "Archives",
      icon: <ArchiveIcon />,
    },
    {
      accent: { light: "#5343c7", dark: "#a692ff" },
      id: EApplicationGroupId.CONFIGS,
      label: "Configs",
      icon: <SettingsApplicationsIcon />,
    },
    {
      accent: { light: "#b22747", dark: "#f87887" },
      id: EApplicationGroupId.DIALOGS,
      label: "Dialogs",
      icon: <ForumIcon />,
    },
    {
      accent: { light: "#986a13", dark: "#e9bd62" },
      id: EApplicationGroupId.ENVIRONMENT,
      label: "Environment",
      icon: <WbSunnyIcon />,
    },
    {
      accent: { light: "#20733d", dark: "#43d37a" },
      id: EApplicationGroupId.GAMEDATA,
      label: "Gamedata",
      icon: <FactCheckIcon />,
    },
    {
      accent: { light: "#b24422", dark: "#ff875b" },
      id: EApplicationGroupId.GAMEPLAY,
      label: "Gameplay",
      icon: <SportsEsportsIcon />,
    },
    {
      accent: { light: "#8a5a3b", dark: "#d8a77d" },
      id: EApplicationGroupId.LEVEL,
      label: "Level",
      icon: <TerrainIcon />,
    },
    {
      accent: { light: "#77634c", dark: "#cfb18f" },
      id: EApplicationGroupId.MATERIALS,
      label: "Materials",
      icon: <LayersIcon />,
    },
    {
      accent: { light: "#a53679", dark: "#f07ec3" },
      id: EApplicationGroupId.PARTICLES,
      label: "Particles",
      icon: <BlurOnIcon />,
    },
    {
      accent: { light: "#08778a", dark: "#2bd0df" },
      id: EApplicationGroupId.SCRIPTS,
      label: "Scripts",
      icon: <CodeIcon />,
    },
    {
      accent: { light: "#6548a3", dark: "#b99aea" },
      id: EApplicationGroupId.SHADERS,
      label: "Shaders",
      icon: <GradientIcon />,
    },
    {
      accent: { light: "#197f78", dark: "#62cfc6" },
      id: EApplicationGroupId.SOUNDS,
      label: "Sounds",
      icon: <GraphicEqIcon />,
    },
    {
      accent: { light: "#59730c", dark: "#a4d83b" },
      id: EApplicationGroupId.SPAWNS,
      label: "Spawns",
      icon: <MapIcon />,
    },
    {
      accent: { light: "#8934c4", dark: "#d087ff" },
      id: EApplicationGroupId.SPRITES,
      label: "Sprites",
      icon: <ImageIcon />,
    },
    {
      accent: { light: "#896400", dark: "#f3c53d" },
      id: EApplicationGroupId.TEXTURES,
      label: "Textures",
      icon: <TextureIcon />,
    },
    {
      accent: { light: "#007b64", dark: "#30d6af" },
      id: EApplicationGroupId.TRANSLATIONS,
      label: "Translations",
      icon: <TranslateIcon />,
    },
    {
      accent: { light: "#006faa", dark: "#39b8ff" },
      id: EApplicationGroupId.VISUALS,
      label: "Visuals",
      icon: <ViewInArIcon />,
    },
  ],
  findApplicationByPath: (pathname: string): Nullable<IApplicationDescriptor> =>
    APPLICATION_CATALOG.applications.find(
      (application: IApplicationDescriptor) =>
        pathname === application.path || pathname.startsWith(`${application.path}/`)
    ) ?? null,
  findApplicationById: (id: string): Nullable<IApplicationDescriptor> =>
    APPLICATION_CATALOG.applications.find((application: IApplicationDescriptor) => application.id === id) ?? null,
  findApplicationGroupById: (id: string): Nullable<IApplicationGroup> =>
    APPLICATION_CATALOG.groups.find((group: IApplicationGroup) => group.id === id) ?? null,
};
