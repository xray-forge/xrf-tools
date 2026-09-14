import { default as ArchiveIcon } from "@mui/icons-material/Archive";
import { default as FactCheckIcon } from "@mui/icons-material/FactCheck";
import { default as ForumIcon } from "@mui/icons-material/Forum";
import { default as ImageIcon } from "@mui/icons-material/Image";
import { default as MapIcon } from "@mui/icons-material/Map";
import { default as SettingsApplicationsIcon } from "@mui/icons-material/SettingsApplications";
import { default as SportsEsportsIcon } from "@mui/icons-material/SportsEsports";
import { default as SwapHorizIcon } from "@mui/icons-material/SwapHoriz";
import { default as TerrainIcon } from "@mui/icons-material/Terrain";
import { default as TextureIcon } from "@mui/icons-material/Texture";
import { default as TranslateIcon } from "@mui/icons-material/Translate";
import { default as ViewInArIcon } from "@mui/icons-material/ViewInAr";

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
import { GAMEDATA_VERIFIER_APPLICATION } from "@/applications/gamedata-verifier";
import { INFO_PORTIONS_EXPLORER_APPLICATION } from "@/applications/info-portions-explorer";
import { LEVEL_COMPILER_APPLICATION } from "@/applications/level-compiler";
import { LEVEL_DECOMPILER_APPLICATION } from "@/applications/level-decompiler";
import { LEVEL_EDITOR_APPLICATION } from "@/applications/level-editor";
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
import { VISUALS_EXPLORER_APPLICATION } from "@/applications/visuals-explorer";
import { VISUALS_SEQUENCER_APPLICATION } from "@/applications/visuals-sequencer";
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
    EXPORTS_EXPLORER_APPLICATION,
    GAMEDATA_VERIFIER_APPLICATION,
    CHARACTERS_EXPLORER_APPLICATION,
    INFO_PORTIONS_EXPLORER_APPLICATION,
    TASKS_EXPLORER_APPLICATION,
    LEVEL_EDITOR_APPLICATION,
    LEVEL_COMPILER_APPLICATION,
    LEVEL_DECOMPILER_APPLICATION,
    SPAWN_EDITOR_APPLICATION,
    SPAWN_PACKER_APPLICATION,
    SPAWN_UNPACKER_APPLICATION,
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
  ],
  groups: [
    {
      accent: { light: "#24c63a", dark: "#6faf5c" },
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
      accent: { light: "#08778a", dark: "#2bd0df" },
      id: EApplicationGroupId.EXPORTS,
      label: "Exports",
      icon: <SwapHorizIcon />,
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
