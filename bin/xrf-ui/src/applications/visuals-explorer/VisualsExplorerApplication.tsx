import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useState } from "react";

import { VISUAL_EXPLORER_PANELS } from "@/applications/visuals-explorer/components/panels/visual-explorer-panels";
import { VisualsMenu } from "@/applications/visuals-explorer/components/tree/VisualsMenu";
import { VisualsExplorerOpenForm } from "@/applications/visuals-explorer/components/VisualsExplorerOpenForm";
import { VisualsBrowseService } from "@/applications/visuals-explorer/services/browse";
import { VisualsService } from "@/applications/visuals-explorer/services/visuals";
import { ApplicationLoader } from "@/core/shell/loading/ApplicationLoader";
import { VisualPreviewLayout } from "@/core/visuals/components/preview/VisualPreviewLayout";
import { IOpenVisual } from "@/core/visuals/services";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

export interface IVisualsExplorerApplicationProps extends BaseComponentProps {}

/**
 * Browse a world of visuals, or look at one model on its own.
 *
 * The layout is mounted for as long as the application is: a load shows in the toolbar and over the viewport instead of
 * replacing the screen, so clicking through a tree keeps the tree, the camera and the webgl context rather than tearing
 * all three down per model. Only the very first provisioning, before anything can be shown, gets the loader.
 */
export function VisualsExplorerApplication({
  "data-testid": dataTestId = "visuals-explorer-application",
  id,
  className,
}: IVisualsExplorerApplicationProps = {}): ReactElement {
  const visualsService: VisualsService = useInjection(VisualsService);
  const browseService: VisualsBrowseService = useInjection(VisualsBrowseService);

  const [isPickerOpen, setPickerOpen] = useState(false);

  const visual: Nullable<IOpenVisual> = visualsService.visual.value;
  const isBrowsing: boolean = browseService.isBrowsing;

  const onOpen = useCallback(() => setPickerOpen(true), []);

  const onFinished = useCallback(() => setPickerOpen(false), []);

  /** Promotes a single-model session to a browsed one, rooted where the model sits. */
  const onBrowse = useCallback(() => {
    const root: Nullable<string> = visualsService.containingRoot;

    if (root) {
      void browseService.openRoot(root);
    }
  }, [browseService, visualsService.containingRoot]);

  if (!visualsService.isReady) {
    return <ApplicationLoader />;
  }

  if (isPickerOpen || (!visual && !isBrowsing)) {
    return <VisualsExplorerOpenForm onFinished={onFinished} />;
  }

  return (
    <VisualPreviewLayout
      data-testid={dataTestId}
      id={id}
      className={className}
      model={visual?.views ?? null}
      subtitle={visualsService.sourceLabel ?? undefined}
      panels={VISUAL_EXPLORER_PANELS}
      textures={visualsService.textures}
      hasMotions={visualsService.hasMotions}
      highlightedJoint={visualsService.highlightedJoint}
      tree={isBrowsing ? <VisualsMenu /> : undefined}
      isLoading={visualsService.visual.isLoading}
      error={visualsService.visual.error?.message}
      onOpen={onOpen}
      onBrowse={isBrowsing ? undefined : onBrowse}
    />
  );
}
