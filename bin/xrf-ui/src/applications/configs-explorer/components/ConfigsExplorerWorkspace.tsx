import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useMemo } from "react";

import { createConfigsExplorerPanels } from "@/applications/configs-explorer/components/panels/configs-explorer-panels";
import { ConfigsProjectDescriptor } from "@/core/bindings/types/xrf-app";
import { ConfigsDocumentView } from "@/core/ltx/components/ConfigsDocumentView";
import { ConfigsDocumentService } from "@/core/ltx/services/document";
import { ConfigsProjectService } from "@/core/ltx/services/project";
import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { EditorToolbarLocation } from "@/core/shell/editor/EditorToolbarLocation";
import { IEditorPanel, useEditorPanels, useEditorStatus } from "@/core/shell/editor-shell";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

/**
 * The browsing session: the project tree, and whichever config is open beside it.
 */
export function ConfigsExplorerWorkspace({
  "data-testid": dataTestId = "configs-explorer-workspace",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const projectService: ConfigsProjectService = useInjection(ConfigsProjectService);
  const documentService: ConfigsDocumentService = useInjection(ConfigsDocumentService);

  const project: Nullable<ConfigsProjectDescriptor> = projectService.project.value;
  const panels: Array<IEditorPanel> = useMemo(() => createConfigsExplorerPanels(), []);

  const onBack = useCallback(() => {
    documentService.clear();

    void projectService.close();
  }, [documentService, projectService]);

  useEditorPanels(() => panels, [panels]);

  // What the session is, rather than what the open document is: the document names itself in the toolbar.
  useEditorStatus(
    useMemo(
      () =>
        project
          ? [
              `${project.inventory.files.length} configs`,
              project.isDltx ? "dltx" : "ltx",
              ...(project.declaredSchemes.length ? [`${project.declaredSchemes.length} schemes`] : []),
            ]
          : [],
      [project]
    )
  );

  return (
    <EditorLayout
      data-testid={dataTestId}
      id={id}
      className={className}
      toolbar={
        <EditorToolbar
          subtitle={
            documentService.selected ? <EditorToolbarLocation location={{ path: documentService.selected }} /> : undefined
          }
          onBack={onBack}
        />
      }
    >
      <ConfigsDocumentView />
    </EditorLayout>
  );
}
