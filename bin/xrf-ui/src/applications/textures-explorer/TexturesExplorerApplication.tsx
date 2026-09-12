import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useState } from "react";

import { ApplicationLoader } from "@/core/shell/loading/ApplicationLoader";
import { TextureCatalogService } from "@/core/textures/services/catalog";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { TexturesExplorerOpenForm } from "./components/TexturesExplorerOpenForm";
import { TexturesExplorerWorkspace } from "./components/TexturesExplorerWorkspace";

/**
 * Browse a root set of textures, or inspect one texture on its own.
 */
export function TexturesExplorerApplication({
  "data-testid": dataTestId = "textures-explorer-application",
}: BaseComponentProps): ReactElement {
  const catalogService: TextureCatalogService = useInjection(TextureCatalogService);
  const selectionService: TextureSelectionService = useInjection(TextureSelectionService);
  const isOpen: boolean = catalogService.isBrowsing || selectionService.selected.value !== null;

  const [isPickerOpen, setPickerOpen] = useState<boolean>(false);

  const onFinished = useCallback(() => setPickerOpen(false), []);

  if (!catalogService.isReady) {
    return <ApplicationLoader />;
  }

  if (isPickerOpen || !isOpen) {
    return <TexturesExplorerOpenForm data-testid={dataTestId} onFinished={onFinished} />;
  }

  return <TexturesExplorerWorkspace data-testid={dataTestId} />;
}
