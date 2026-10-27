import { Nullable } from "@xrf/types";
import { ArchiveIcon } from "@xrf/ui/icons/archive";
import { BlurOnIcon } from "@xrf/ui/icons/blur-on";
import { CodeIcon } from "@xrf/ui/icons/code";
import { FactCheckIcon } from "@xrf/ui/icons/fact-check";
import { ForumIcon } from "@xrf/ui/icons/forum";
import { GradientIcon } from "@xrf/ui/icons/gradient";
import { GraphicEqIcon } from "@xrf/ui/icons/graphic-eq";
import { ImageIcon } from "@xrf/ui/icons/image";
import { LayersIcon } from "@xrf/ui/icons/layers";
import { MapIcon } from "@xrf/ui/icons/map";
import { SettingsApplicationsIcon } from "@xrf/ui/icons/settings-applications";
import { SportsEsportsIcon } from "@xrf/ui/icons/sports-esports";
import { TerrainIcon } from "@xrf/ui/icons/terrain";
import { TextureIcon } from "@xrf/ui/icons/texture";
import { TranslateIcon } from "@xrf/ui/icons/translate";
import { ViewInArIcon } from "@xrf/ui/icons/view-in-ar";
import { WbSunnyIcon } from "@xrf/ui/icons/wb-sunny";
import { CATALOG_ACCENT } from "@xrf/ui/theme";

import { EToolGroupId, IToolDescriptor, IToolGroup } from "@/core/routing/tool";
import { ARCHIVES_EXPLORER_TOOL } from "@/tools/archives-explorer";
import { ARCHIVES_PACKER_TOOL } from "@/tools/archives-packer";
import { ARCHIVES_PATCHER_TOOL } from "@/tools/archives-patcher";
import { ARCHIVES_UNPACKER_TOOL } from "@/tools/archives-unpacker";
import { CHARACTERS_EXPLORER_TOOL } from "@/tools/characters-explorer";
import { CONFIGS_EXPLORER_TOOL } from "@/tools/configs-explorer";
import { CONFIGS_FORMATTER_TOOL } from "@/tools/configs-formatter";
import { CONFIGS_VERIFIER_TOOL } from "@/tools/configs-verifier";
import { DIALOGS_EDITOR_TOOL } from "@/tools/dialogs-editor";
import { EXPORTS_EXPLORER_TOOL } from "@/tools/exports-explorer";
import { GAME_MATERIALS_EDITOR_TOOL } from "@/tools/game-materials-editor";
import { GAMEDATA_COMPARER_TOOL } from "@/tools/gamedata-comparer";
import { GAMEDATA_VERIFIER_TOOL } from "@/tools/gamedata-verifier";
import { INFO_PORTIONS_EXPLORER_TOOL } from "@/tools/info-portions-explorer";
import { LEVEL_AI_COMPILER_TOOL } from "@/tools/level-ai-compiler";
import { LEVEL_COMPILER_TOOL } from "@/tools/level-compiler";
import { LEVEL_DECOMPILER_TOOL } from "@/tools/level-decompiler";
import { LEVEL_DETAILS_COMPILER_TOOL } from "@/tools/level-details-compiler";
import { LEVEL_EDITOR_TOOL } from "@/tools/level-editor";
import { LEVEL_VIEWER_TOOL } from "@/tools/level-viewer";
import { LIGHT_ANIMATIONS_EDITOR_TOOL } from "@/tools/light-animations-editor";
import { LUA_EXPORTS_EXPLORER_TOOL } from "@/tools/lua-exports-explorer";
import { LUA_FORMATTER_TOOL } from "@/tools/lua-formatter";
import { LUA_VERIFIER_TOOL } from "@/tools/lua-verifier";
import { MINIMAP_EDITOR_TOOL } from "@/tools/minimap-editor";
import { OBJECT_LIBRARY_EDITOR_TOOL } from "@/tools/object-library-editor";
import { PARTICLES_EDITOR_TOOL } from "@/tools/particles-editor";
import { PARTICLES_EXPLORER_TOOL } from "@/tools/particles-explorer";
import { POSTPROCESS_EDITOR_TOOL } from "@/tools/postprocess-editor";
import { SHADERS_EDITOR_TOOL } from "@/tools/shaders-editor";
import { SHADERS_EXPLORER_TOOL } from "@/tools/shaders-explorer";
import { SOUND_ENVIRONMENTS_EDITOR_TOOL } from "@/tools/sound-environments-editor";
import { SOUNDS_EDITOR_TOOL } from "@/tools/sounds-editor";
import { SOUNDS_EXPLORER_TOOL } from "@/tools/sounds-explorer";
import { SPAWN_COMPILER_TOOL } from "@/tools/spawn-compiler";
import { SPAWN_EDITOR_TOOL } from "@/tools/spawn-editor";
import { SPAWN_PACKER_TOOL } from "@/tools/spawn-packer";
import { SPAWN_UNPACKER_TOOL } from "@/tools/spawn-unpacker";
import { SPRITE_DESCRIPTION_EDITOR_TOOL } from "@/tools/sprite-description-editor";
import { SPRITE_DESCRIPTION_PACKER_TOOL } from "@/tools/sprite-description-packer";
import { SPRITE_DESCRIPTION_UNPACKER_TOOL } from "@/tools/sprite-description-unpacker";
import { SPRITE_EQUIPMENT_EDITOR_TOOL } from "@/tools/sprite-equipment-editor";
import { SPRITE_EQUIPMENT_PACKER_TOOL } from "@/tools/sprite-equipment-packer";
import { SPRITE_EQUIPMENT_UNPACKER_TOOL } from "@/tools/sprite-equipment-unpacker";
import { TASKS_EXPLORER_TOOL } from "@/tools/tasks-explorer";
import { TEXTURES_EDITOR_TOOL } from "@/tools/textures-editor";
import { TEXTURES_EXPLORER_TOOL } from "@/tools/textures-explorer";
import { TRANSLATIONS_BUILDER_TOOL } from "@/tools/translations-builder";
import { TRANSLATIONS_EDITOR_TOOL } from "@/tools/translations-editor";
import { TRANSLATIONS_FORMATTER_TOOL } from "@/tools/translations-formatter";
import { TRANSLATIONS_PARSER_TOOL } from "@/tools/translations-parser";
import { TRANSLATIONS_VERIFIER_TOOL } from "@/tools/translations-verifier";
import { VISUALS_CONVERTER_TOOL } from "@/tools/visuals-converter";
import { VISUALS_EDITOR_TOOL } from "@/tools/visuals-editor";
import { VISUALS_EXPLORER_TOOL } from "@/tools/visuals-explorer";
import { VISUALS_SEQUENCER_TOOL } from "@/tools/visuals-sequencer";
import { WEATHER_EDITOR_TOOL } from "@/tools/weather-editor";

export interface IToolCatalog {
  readonly tools: ReadonlyArray<IToolDescriptor>;
  readonly groups: ReadonlyArray<IToolGroup>;
  findToolByPath(pathname: string): Nullable<IToolDescriptor>;
  findToolById(id: string): Nullable<IToolDescriptor>;
  findToolGroupById(id: string): Nullable<IToolGroup>;
}

export const TOOL_CATALOG: IToolCatalog = {
  tools: [
    ARCHIVES_EXPLORER_TOOL,
    ARCHIVES_PACKER_TOOL,
    ARCHIVES_PATCHER_TOOL,
    ARCHIVES_UNPACKER_TOOL,
    CONFIGS_EXPLORER_TOOL,
    CONFIGS_VERIFIER_TOOL,
    CONFIGS_FORMATTER_TOOL,
    DIALOGS_EDITOR_TOOL,
    WEATHER_EDITOR_TOOL,
    LIGHT_ANIMATIONS_EDITOR_TOOL,
    POSTPROCESS_EDITOR_TOOL,
    GAMEDATA_VERIFIER_TOOL,
    GAMEDATA_COMPARER_TOOL,
    CHARACTERS_EXPLORER_TOOL,
    INFO_PORTIONS_EXPLORER_TOOL,
    TASKS_EXPLORER_TOOL,
    LEVEL_EDITOR_TOOL,
    LEVEL_COMPILER_TOOL,
    LEVEL_DECOMPILER_TOOL,
    LEVEL_VIEWER_TOOL,
    MINIMAP_EDITOR_TOOL,
    LEVEL_AI_COMPILER_TOOL,
    LEVEL_DETAILS_COMPILER_TOOL,
    GAME_MATERIALS_EDITOR_TOOL,
    PARTICLES_EXPLORER_TOOL,
    PARTICLES_EDITOR_TOOL,
    EXPORTS_EXPLORER_TOOL,
    LUA_EXPORTS_EXPLORER_TOOL,
    LUA_VERIFIER_TOOL,
    LUA_FORMATTER_TOOL,
    SHADERS_EXPLORER_TOOL,
    SHADERS_EDITOR_TOOL,
    SOUNDS_EXPLORER_TOOL,
    SOUNDS_EDITOR_TOOL,
    SOUND_ENVIRONMENTS_EDITOR_TOOL,
    SPAWN_EDITOR_TOOL,
    SPAWN_PACKER_TOOL,
    SPAWN_UNPACKER_TOOL,
    SPAWN_COMPILER_TOOL,
    SPRITE_EQUIPMENT_EDITOR_TOOL,
    SPRITE_EQUIPMENT_PACKER_TOOL,
    SPRITE_EQUIPMENT_UNPACKER_TOOL,
    SPRITE_DESCRIPTION_EDITOR_TOOL,
    SPRITE_DESCRIPTION_PACKER_TOOL,
    SPRITE_DESCRIPTION_UNPACKER_TOOL,
    TEXTURES_EXPLORER_TOOL,
    TEXTURES_EDITOR_TOOL,
    TRANSLATIONS_EDITOR_TOOL,
    TRANSLATIONS_PARSER_TOOL,
    TRANSLATIONS_BUILDER_TOOL,
    TRANSLATIONS_VERIFIER_TOOL,
    TRANSLATIONS_FORMATTER_TOOL,
    VISUALS_EXPLORER_TOOL,
    VISUALS_SEQUENCER_TOOL,
    VISUALS_CONVERTER_TOOL,
    VISUALS_EDITOR_TOOL,
    OBJECT_LIBRARY_EDITOR_TOOL,
  ],
  groups: [
    {
      accent: CATALOG_ACCENT.archives,
      id: EToolGroupId.ARCHIVES,
      label: "Archives",
      icon: <ArchiveIcon />,
    },
    {
      accent: CATALOG_ACCENT.configs,
      id: EToolGroupId.CONFIGS,
      label: "Configs",
      icon: <SettingsApplicationsIcon />,
    },
    {
      accent: CATALOG_ACCENT.dialogs,
      id: EToolGroupId.DIALOGS,
      label: "Dialogs",
      icon: <ForumIcon />,
    },
    {
      accent: CATALOG_ACCENT.environment,
      id: EToolGroupId.ENVIRONMENT,
      label: "Environment",
      icon: <WbSunnyIcon />,
    },
    {
      accent: CATALOG_ACCENT.gamedata,
      id: EToolGroupId.GAMEDATA,
      label: "Gamedata",
      icon: <FactCheckIcon />,
    },
    {
      accent: CATALOG_ACCENT.gameplay,
      id: EToolGroupId.GAMEPLAY,
      label: "Gameplay",
      icon: <SportsEsportsIcon />,
    },
    {
      accent: CATALOG_ACCENT.level,
      id: EToolGroupId.LEVEL,
      label: "Level",
      icon: <TerrainIcon />,
    },
    {
      accent: CATALOG_ACCENT.materials,
      id: EToolGroupId.MATERIALS,
      label: "Materials",
      icon: <LayersIcon />,
    },
    {
      accent: CATALOG_ACCENT.particles,
      id: EToolGroupId.PARTICLES,
      label: "Particles",
      icon: <BlurOnIcon />,
    },
    {
      accent: CATALOG_ACCENT.scripts,
      id: EToolGroupId.SCRIPTS,
      label: "Scripts",
      icon: <CodeIcon />,
    },
    {
      accent: CATALOG_ACCENT.shaders,
      id: EToolGroupId.SHADERS,
      label: "Shaders",
      icon: <GradientIcon />,
    },
    {
      accent: CATALOG_ACCENT.sounds,
      id: EToolGroupId.SOUNDS,
      label: "Sounds",
      icon: <GraphicEqIcon />,
    },
    {
      accent: CATALOG_ACCENT.spawns,
      id: EToolGroupId.SPAWNS,
      label: "Spawns",
      icon: <MapIcon />,
    },
    {
      accent: CATALOG_ACCENT.sprites,
      id: EToolGroupId.SPRITES,
      label: "Sprites",
      icon: <ImageIcon />,
    },
    {
      accent: CATALOG_ACCENT.textures,
      id: EToolGroupId.TEXTURES,
      label: "Textures",
      icon: <TextureIcon />,
    },
    {
      accent: CATALOG_ACCENT.translations,
      id: EToolGroupId.TRANSLATIONS,
      label: "Translations",
      icon: <TranslateIcon />,
    },
    {
      accent: CATALOG_ACCENT.visuals,
      id: EToolGroupId.VISUALS,
      label: "Visuals",
      icon: <ViewInArIcon />,
    },
  ],
  findToolByPath: (pathname: string): Nullable<IToolDescriptor> =>
    TOOL_CATALOG.tools.find(
      (tool: IToolDescriptor) => pathname === tool.path || pathname.startsWith(`${tool.path}/`)
    ) ?? null,
  findToolById: (id: string): Nullable<IToolDescriptor> =>
    TOOL_CATALOG.tools.find((tool: IToolDescriptor) => tool.id === id) ?? null,
  findToolGroupById: (id: string): Nullable<IToolGroup> =>
    TOOL_CATALOG.groups.find((group: IToolGroup) => group.id === id) ?? null,
};
