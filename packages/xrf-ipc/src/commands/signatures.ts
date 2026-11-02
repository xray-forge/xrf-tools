// Auto-generated rust bindings. Do not edit it manually.

import { IArchivesCommandSignatures } from "#/commands/archives";
import { IArchivesBulkRouteSignatures } from "#/commands/archives-bulk";
import { IAssetsCommandSignatures } from "#/commands/assets";
import { IAssetsBulkRouteSignatures } from "#/commands/assets-bulk";
import { IConfigsCommandSignatures } from "#/commands/configs";
import { IDialogsCommandSignatures } from "#/commands/dialogs";
import { IExportsCommandSignatures } from "#/commands/exports";
import { IGamedataCommandSignatures } from "#/commands/gamedata";
import { IJobsCommandSignatures } from "#/commands/jobs";
import { ILevelsCommandSignatures } from "#/commands/levels";
import { ILevelsBulkRouteSignatures } from "#/commands/levels-bulk";
import { IRenderCommandSignatures } from "#/commands/render";
import { ISessionsCommandSignatures } from "#/commands/sessions";
import { ISpawnCommandSignatures } from "#/commands/spawn";
import { ISpriteEquipmentCommandSignatures } from "#/commands/sprite-equipment";
import { ISpriteEquipmentBulkRouteSignatures } from "#/commands/sprite-equipment-bulk";
import { ISystemCommandSignatures } from "#/commands/system";
import { ITexturesCommandSignatures } from "#/commands/textures";
import { ITexturesBulkRouteSignatures } from "#/commands/textures-bulk";
import { ITranslationsCommandSignatures } from "#/commands/translations";
import { ITransportCommandSignatures } from "#/commands/transport";
import { IVisualsCommandSignatures } from "#/commands/visuals";

/** Every command any plugin answers, raw ones included, keyed by its wire name. */
export interface IIpcCommandSignatures
  extends
    IArchivesCommandSignatures,
    IAssetsCommandSignatures,
    IConfigsCommandSignatures,
    IDialogsCommandSignatures,
    IExportsCommandSignatures,
    IGamedataCommandSignatures,
    IJobsCommandSignatures,
    ILevelsCommandSignatures,
    IRenderCommandSignatures,
    ISessionsCommandSignatures,
    ISpawnCommandSignatures,
    ISpriteEquipmentCommandSignatures,
    ISystemCommandSignatures,
    ITexturesCommandSignatures,
    ITranslationsCommandSignatures,
    ITransportCommandSignatures,
    IVisualsCommandSignatures {}

/** Every bulk route any plugin serves, keyed by its route. */
export interface IIpcBulkRouteSignatures
  extends
    IArchivesBulkRouteSignatures,
    IAssetsBulkRouteSignatures,
    ILevelsBulkRouteSignatures,
    ISpriteEquipmentBulkRouteSignatures,
    ITexturesBulkRouteSignatures {}
