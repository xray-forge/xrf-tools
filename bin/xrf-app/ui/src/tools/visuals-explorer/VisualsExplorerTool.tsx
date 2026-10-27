import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { EditorToolbarLocation } from "@xrf/ui/editor/toolbar";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement, useCallback, useMemo, useState } from "react";

import { ToolLoader } from "@/core/shell/chrome/ToolLoader";
import { VisualPreviewLayout } from "@/core/visuals/components/preview/VisualPreviewLayout";
import { IOpenVisual } from "@/core/visuals/services";
import { toVisualSessionLocation } from "@/tools/visuals-explorer/lib/visual-location";
import { VisualsService } from "@/tools/visuals-explorer/services/visuals";
import { VisualsBrowseService } from "@/tools/visuals-explorer/services/visuals-browse";

import { VISUALS_EXPLORER_PANELS } from "./components/panels";
import { VisualsMenu } from "./components/tree";
import { VisualsExplorerOpenForm } from "./components/VisualsExplorerOpenForm";

/**
 * Browse a tree of visuals, or look at one model on its own.
 *
 * The layout is mounted for as long as the tool is: a load shows in the toolbar and over the viewport instead of
 * replacing the screen, so clicking through a tree keeps the tree, the camera and the webgl context rather than tearing
 * all three down per model. Only the very first provisioning, before anything can be shown, gets the loader.
 */
export function VisualsExplorerTool({
  "data-testid": dataTestId = "visuals-explorer-tool",
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
    return <ToolLoader />;
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
