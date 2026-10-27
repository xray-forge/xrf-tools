import { inject, Injectable } from "@wirestate/core";
import { EAssetSource } from "@xrf/ipc/types/xrf-app-core";

import { createRoots } from "@/core/assets/lib";
import { VisualInspectionService } from "@/core/visuals/services/visual-inspection";
import { VisualLoadService } from "@/core/visuals/services/visual-load";

/**
 * An inspection over the loader alone, for panel tests that need a visual open and no tool around it.
 */
@Injectable()
export class MockVisualInspectionService extends VisualInspectionService {
  public constructor(loadService: VisualLoadService = inject(VisualLoadService)) {
    super(loadService);
  }

  /** Nothing: a fixture that only inspects offers no bone marking. */
  public get boneControls(): null {
    return null;
  }

  /**
   * Opens a loose visual as the explorer does: centred on the file, so its own tree is searched.
   *
   * @param path - The file on disk.
   * @returns Once the loader settled.
   */
  public async openFile(path: string): Promise<void> {
    await this.loadService.load({ kind: EAssetSource.FILE, path }, createRoots([], path));
  }
}
