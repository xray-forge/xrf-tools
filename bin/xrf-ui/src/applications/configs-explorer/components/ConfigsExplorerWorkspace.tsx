import { default as AccountTreeIcon } from "@mui/icons-material/AccountTree";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useMemo } from "react";

import { createConfigsExplorerPanels } from "@/applications/configs-explorer/components/panels/configs-explorer-panels";
import { ConfigsProjectDescriptor } from "@/core/bindings/types/xrf-app";
import { ConfigsDocumentView } from "@/core/ltx/components/ConfigsDocumentView";
import { ConfigsDocumentService, EConfigsDocumentMode } from "@/core/ltx/services/document";
import { ConfigsProjectService } from "@/core/ltx/services/project";
import { ConfigsResolvedService } from "@/core/ltx/services/resolved";
import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { EditorToolbarLocation } from "@/core/shell/editor/EditorToolbarLocation";
import { EditorViewToggle } from "@/core/shell/editor/EditorViewToggle";
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
  const resolvedService: ConfigsResolvedService = useInjection(ConfigsResolvedService);

  const project: Nullable<ConfigsProjectDescriptor> = projectService.project.value;
  const panels: Array<IEditorPanel> = useMemo(() => createConfigsExplorerPanels(), []);

  const selected: Nullable<string> = documentService.selected;
  const entry: Nullable<string> = documentService.entry;
  const isResolved: boolean = documentService.mode === EConfigsDocumentMode.RESOLVED;

  const onBack = useCallback(() => {
    documentService.clear();
    resolvedService.clear();

    void projectService.close();
  }, [documentService, projectService, resolvedService]);

  const onToggleMode = useCallback(
    () => documentService.setMode(isResolved ? EConfigsDocumentMode.AUTHORED : EConfigsDocumentMode.RESOLVED),
    [documentService, isResolved]
  );

  // Resolved on entering the view rather than on opening the file: resolving a root is seconds of work on an
  // installation, and most of what a person opens they only ever read as written.
  useEffect(() => {
    if (isResolved && entry) {
      void resolvedService.open(entry);
    }
  }, [entry, isResolved, resolvedService]);

  // An included config is one of thousands of sections in its entry point's resolution, so the view opens narrowed to
  // what this file itself declared. An entry point is its own document and narrows to nothing.
  useEffect(() => {
    resolvedService.narrowTo(selected && selected !== entry ? selected : null);
  }, [entry, resolvedService, selected]);

  useEditorPanels(() => panels, [panels]);

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
          subtitle={selected ? <EditorToolbarLocation location={{ path: selected }} /> : undefined}
          actions={
            <EditorViewToggle
              label={"Resolved"}
              description={
                isResolved
                  ? `Showing what ${entry ?? "this config"} resolves to`
                  : "Show what this config resolves to, with every value's origin"
              }
              icon={<AccountTreeIcon />}
              isOn={isResolved}
              // Nothing resolves a config no entry point reaches, which under a patch dialect is an attachment.
              isDisabled={!entry}
              unavailableTitle={selected && !entry ? "No entry point resolves this config" : undefined}
              onToggle={onToggleMode}
            />
          }
          onBack={onBack}
        />
      }
    >
      <ConfigsDocumentView />
    </EditorLayout>
  );
}
