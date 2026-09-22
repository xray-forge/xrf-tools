import { inject, Injectable } from "@wirestate/core";
import { BoundAction, Computed, Observable } from "@wirestate/mobx";
import { Nullable } from "@xrf/types";

import { SpriteEquipmentEditorService } from "@/applications/sprite-equipment-editor/services/editor";
import {
  IEquipmentLayout,
  isCellWithin,
  isSameCell,
  TEquipmentCell,
  toEquipmentLayout,
} from "@/core/sprite-equipment/lib";

/**
 * The lattice over the open sheet, and which cell of it is being explained.
 */
@Injectable()
export class EquipmentGridService {
  @Observable()
  private picked: Nullable<TEquipmentCell> = null;

  public constructor(
    private readonly editorService: SpriteEquipmentEditorService = inject(SpriteEquipmentEditorService)
  ) {}

  @Computed()
  public get layout(): Nullable<IEquipmentLayout> {
    const sprite = this.editorService.spriteImage.value;

    return sprite
      ? toEquipmentLayout(
          sprite.image.width,
          sprite.image.height,
          this.editorService.gridSize,
          sprite.metadata.occupants
        )
      : null;
  }

  @Computed()
  public get selectedCell(): Nullable<TEquipmentCell> {
    const layout: Nullable<IEquipmentLayout> = this.layout;

    return layout && this.picked && isCellWithin(layout.grid, this.picked) ? this.picked : null;
  }

  /**
   * Explains a cell, or stops explaining it when it is the one already open.
   *
   * @param cell - Cell to explain, or null to explain nothing.
   */
  @BoundAction()
  public selectCell(cell: Nullable<TEquipmentCell>): void {
    this.picked = isSameCell(this.picked, cell) ? null : cell;
  }

  /**
   * Explains a cell, whichever one was open before.
   *
   * @param cell - Cell to explain.
   */
  @BoundAction()
  public revealCell(cell: TEquipmentCell): void {
    this.picked = cell;
  }

  @BoundAction()
  public clearSelection(): void {
    this.picked = null;
  }
}
