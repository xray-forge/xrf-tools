import { AsyncState } from "@xrf/async";
import { EquipmentSpriteMetadata } from "@xrf/ipc/types/xrf-app-sprite-equipment";

import {
  IOpenEquipmentSprite,
  SpriteEquipmentEditorService,
} from "@/applications/sprite-equipment-editor/services/editor";
import { mockCommandAnswer } from "@/core/operations/fixtures/command-operation.mocks";
import { mockHeldSessionRoot } from "@/core/session/fixtures/session-root.mocks";
import { mockEquipmentSpriteMetadata } from "@/core/sprite-equipment/fixtures/sprite.mocks";

/**
 * Puts a sheet and its decoded picture on the editor as a restore would, the opening owned so a close releases it.
 *
 * @param service - Editor to show the sheet.
 * @param metadata - What the backend says of it.
 * @param sessionId - Identity of its opening.
 * @returns What is on screen.
 */
export function mockShownEquipmentSprite(
  service: SpriteEquipmentEditorService,
  metadata: EquipmentSpriteMetadata = mockEquipmentSpriteMetadata(),
  sessionId: string = "fixture-session"
): IOpenEquipmentSprite {
  const shown: IOpenEquipmentSprite = { sessionId, metadata, image: new Image() };

  mockHeldSessionRoot(service.sprite, { revision: 0, sessionId, value: metadata });
  mockCommandAnswer(service.picture, AsyncState.ready(shown));

  return shown;
}
