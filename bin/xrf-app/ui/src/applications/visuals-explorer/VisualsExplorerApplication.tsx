import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback, useMemo, useState } from "react";

import { toVisualSessionLocation } from "@/applications/visuals-explorer/lib/visual-location";
import { VisualsBrowseService } from "@/applications/visuals-explorer/services/browse";
import { VisualsService } from "@/applications/visuals-explorer/services/visuals";
import { EditorToolbarLocation } from "@/core/shell/editor/EditorToolbarLocation";
import { ApplicationLoader } from "@/core/shell/loading/ApplicationLoader";
import { VisualPreviewLayout } from "@/core/visuals/components/preview/VisualPreviewLayout";
import { IOpenVisual } from "@/core/visuals/services";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { VISUALS_EXPLORER_PANELS } from "./components/panels";
import { VisualsMenu } from "./components/tree";
import { VisualsExplorerOpenForm } from "./components/VisualsExplorerOpenForm";

/**
 * Browse a tree of visuals, or look at one model on its own.
 *
 * The layout is mounted for as long as the application is: a load shows in the toolbar and over the viewport instead of
 * replacing the screen, so clicking through a tree keeps the tree, the camera and the webgl context rather than tearing
 * all three down per model. Only the very first provisioning, before anything can be shown, gets the loader.
 */
export function VisualsExplorerApplication({
  "data-testid": dataTestId = "visuals-explorer-application",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const visualsService: VisualsService = useInjection(VisualsService);
  const browseService: VisualsBrowseService = useInjection(VisualsBrowseService);

  const [isPickerOpen, setPickerOpen] = useState(false);

  const visual: Nullable<IOpenVisual> = visualsService.visual.value;
  const isBrowsing: boolean = browseService.isBrowsing;

  const source = visualsService.selected?.source ?? null;
  const location = useMemo(
    () => toVisualSessionLocation(browseService.rootsLabel, source, browseService.visuals.value ?? []),
    [browseService.rootsLabel, source, browseService.visuals.value]
  );

  const onBack = useCallback(() => setPickerOpen(true), []);

  const onDeselect = useCallback(() => void visualsService.close(), [visualsService]);

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
      subtitle={location ? <EditorToolbarLocation location={location} /> : undefined}
      name={visualsService.sourceLabel}
      panels={VISUALS_EXPLORER_PANELS}
      tree={isBrowsing ? <VisualsMenu /> : undefined}
      isLoading={visualsService.visual.isLoading}
      error={visualsService.visual.error?.message}
      onRetry={visualsService.retryOpen}
      onBack={onBack}
      onBrowse={isBrowsing ? undefined : onBrowse}
      onDeselect={isBrowsing ? onDeselect : null}
    />
  );
}
