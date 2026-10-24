// Auto-generated rust bindings. Do not edit it manually.

import { IArchivesCommandSignatures } from "@/core/ipc/commands/archives";
import { IArchivesBulkRouteSignatures } from "@/core/ipc/commands/archives-bulk";
import { IAssetsCommandSignatures } from "@/core/ipc/commands/assets";
import { IAssetsBulkRouteSignatures } from "@/core/ipc/commands/assets-bulk";
import { IConfigsCommandSignatures } from "@/core/ipc/commands/configs";
import { IDialogsCommandSignatures } from "@/core/ipc/commands/dialogs";
import { IExportsCommandSignatures } from "@/core/ipc/commands/exports";
import { IGamedataCommandSignatures } from "@/core/ipc/commands/gamedata";
import { IJobsCommandSignatures } from "@/core/ipc/commands/jobs";
import { ILevelsCommandSignatures } from "@/core/ipc/commands/levels";
import { ILevelsBulkRouteSignatures } from "@/core/ipc/commands/levels-bulk";
import { IRenderCommandSignatures } from "@/core/ipc/commands/render";
import { ISessionsCommandSignatures } from "@/core/ipc/commands/sessions";
import { ISpawnCommandSignatures } from "@/core/ipc/commands/spawn";
import { ISpriteEquipmentCommandSignatures } from "@/core/ipc/commands/sprite-equipment";
import { ISpriteEquipmentBulkRouteSignatures } from "@/core/ipc/commands/sprite-equipment-bulk";
import { ISystemCommandSignatures } from "@/core/ipc/commands/system";
import { ITexturesCommandSignatures } from "@/core/ipc/commands/textures";
import { ITexturesBulkRouteSignatures } from "@/core/ipc/commands/textures-bulk";
import { ITranslationsCommandSignatures } from "@/core/ipc/commands/translations";
import { ITransportCommandSignatures } from "@/core/ipc/commands/transport";
import { IVisualsCommandSignatures } from "@/core/ipc/commands/visuals";

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
