import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import { ILevelHelperOptions } from "@/core/level/lib/objects/level-helper-options";
import { ELevelShading, getLevelShadings, ILevelShadingChoice } from "@/core/level/lib/view/level-shading";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { LevelRenderService } from "@/core/level/services/level-render";
import { LevelViewService } from "@/core/level/services/level-view";
import { isRenderParallaxDrawn } from "@/core/render/lib/features/world/render-parallax-choices";
import { useShellMenus } from "@/core/shell/contributions";
import { IMenuContribution } from "@/core/shell/menu/lib";
import {
  ILevelViewerViewMenuState,
  toLevelViewerFrameEntries,
  toLevelViewerObjectEntries,
} from "@/tools/level-viewer/components/menus/LevelViewerViewMenu.utils";
import { LEVEL_VIEWER_VIEW_MENU } from "@/tools/level-viewer/menus";

/**
 * The level viewer's View menu: the frame's shading and overlays, then the level objects' helpers and which are drawn,
 * each switching what the toolbar's popover for it does.
 */
export function LevelViewerViewMenu(): ReactElement {
  const viewService: LevelViewService = useInjection(LevelViewService);
  const renderService: LevelRenderService = useInjection(LevelRenderService);
  const { settings, features } = useLevelRenderFeatures();

  const options: ILevelViewOptions = viewService.options;
  const helpers: ILevelHelperOptions = viewService.helpers;
  const shading: ELevelShading = renderService.drawing.shading;
  const shadings: ReadonlyArray<ILevelShadingChoice> = getLevelShadings(renderService.drawing.antialiasing);
  const isParallaxDrawn: boolean = isRenderParallaxDrawn(toLevelModeFeature("parallax", settings, features));

  useShellMenus((): Array<IMenuContribution> => {
    const state: ILevelViewerViewMenuState = { options, helpers, shading, shadings, isParallaxDrawn };

    return [
      { section: LEVEL_VIEWER_VIEW_MENU.sections.frame, entries: toLevelViewerFrameEntries(state, viewService) },
      { section: LEVEL_VIEWER_VIEW_MENU.sections.objects, entries: toLevelViewerObjectEntries(state, viewService) },
    ];
  }, [helpers, isParallaxDrawn, options, shading, shadings, viewService]);

  return <></>;
}
