import { default as ArchiveIcon } from "@mui/icons-material/Archive";
import { default as FactCheckIcon } from "@mui/icons-material/FactCheck";
import { default as ForumIcon } from "@mui/icons-material/Forum";
import { default as ImageIcon } from "@mui/icons-material/Image";
import { default as MapIcon } from "@mui/icons-material/Map";
import { default as SettingsApplicationsIcon } from "@mui/icons-material/SettingsApplications";
import { default as SportsEsportsIcon } from "@mui/icons-material/SportsEsports";
import { default as SwapHorizIcon } from "@mui/icons-material/SwapHoriz";
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
      accent: { light: "#a0444f", dark: "#df909a" },
      id: EApplicationGroupId.ARCHIVES,
      label: "Archives",
      icon: <ArchiveIcon />,
    },
    {
      accent: { light: "#4d61b5", dark: "#99acf1" },
      id: EApplicationGroupId.CONFIGS,
      label: "Configs",
      icon: <SettingsApplicationsIcon />,
    },
    {
      accent: { light: "#ab4674", dark: "#e59bbd" },
      id: EApplicationGroupId.DIALOGS,
      label: "Dialogs",
      icon: <ForumIcon />,
    },
    {
      accent: { light: "#12758d", dark: "#78cadd" },
      id: EApplicationGroupId.EXPORTS,
      label: "Exports",
      icon: <SwapHorizIcon />,
    },
    {
      accent: { light: "#526477", dark: "#a4b5c7" },
      id: EApplicationGroupId.GAMEDATA,
      label: "Gamedata",
      icon: <FactCheckIcon />,
    },
    {
      accent: { light: "#ad5145", dark: "#eea092" },
      id: EApplicationGroupId.GAMEPLAY,
      label: "Gameplay",
      icon: <SportsEsportsIcon />,
    },
    {
      accent: { light: "#596f32", dark: "#b2c887" },
      id: EApplicationGroupId.SPAWNS,
      label: "Spawns",
      icon: <MapIcon />,
    },
    {
      accent: { light: "#8052ad", dark: "#bf9de4" },
      id: EApplicationGroupId.SPRITES,
      label: "Sprites",
      icon: <ImageIcon />,
    },
    {
      accent: { light: "#86661f", dark: "#d7bd78" },
      id: EApplicationGroupId.TEXTURES,
      label: "Textures",
      icon: <TextureIcon />,
    },
    {
      accent: { light: "#237963", dark: "#81c8b5" },
      id: EApplicationGroupId.TRANSLATIONS,
      label: "Translations",
      icon: <TranslateIcon />,
    },
    {
      accent: { light: "#2c6ca0", dark: "#88bcea" },
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
