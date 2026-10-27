import { default as ListAltIcon } from "@mui/icons-material/ListAlt";
import { default as RefreshIcon } from "@mui/icons-material/Refresh";
import { Alert } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { formatCount } from "@xrf/format";
import { ExportDescriptor, ExportsProject } from "@xrf/ipc/types/xrf-export";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback, useEffect, useMemo, useState } from "react";

import { ExportsService } from "@/applications/exports-explorer/services/exports.service";
import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { EditorToolbarLocation } from "@/core/shell/editor/EditorToolbarLocation";
import { useEditorBusy } from "@/core/shell/editor-lifecycle";
import { useEditorPanels, useEditorStatus } from "@/core/shell/editor-shell";

import { groupExports, IExportGroup } from "./exports/exports-groups";
import { ExportsMenu } from "./exports/ExportsMenu";
import { ExportsViewer } from "./exports/ExportsViewer";
import { ExportsSaveAction } from "./ExportsSaveAction";

export function ExportsEditor(): ReactElement {
  const exportsService: ExportsService = useInjection(ExportsService);

  const [selectedName, setSelectedName] = useState<Nullable<string>>(null);

  const project: Nullable<ExportsProject> = exportsService.project.value;
  const declarations: Array<ExportDescriptor> = useMemo(() => project?.declarations ?? [], [project?.declarations]);
  const groups: Array<IExportGroup> = useMemo(() => groupExports(declarations), [declarations]);
  const selectedDeclaration: Nullable<ExportDescriptor> =
    declarations.find((declaration: ExportDescriptor) => declaration.name === selectedName) ?? null;
  const isBusy: boolean = exportsService.project.isLoading || exportsService.manifest.isLoading;

  const onSelect = useCallback((name: string): void => setSelectedName(name), []);

  const onDeselect = useCallback((): void => setSelectedName(null), []);

  const onRefresh = useCallback((): void => {
    void exportsService.refreshExportsProject();
  }, [exportsService]);

  // Closing does not navigate: the application shows its own picker again once nothing is open.
  const onClose = useCallback((): void => {
    void exportsService.closeExportsProject();
  }, [exportsService]);

  useEffect(() => {
    if (selectedName && !selectedDeclaration) {
      setSelectedName(null);
    }
  }, [selectedDeclaration, selectedName]);

  useEditorPanels(
    () => [
      {
        icon: <ListAltIcon />,
        id: "declarations",
        isOpenByDefault: true,
        label: "Declarations",
        render: () => (
          <ExportsMenu declarations={declarations} groups={groups} selectedName={selectedName} onSelect={onSelect} />
        ),
        side: "left",
      },
    ],
    [declarations, groups, onSelect, selectedName]
  );

  useEditorBusy(isBusy);
  useEditorStatus([
    `${formatCount(declarations.length)} ${declarations.length === 1 ? "export" : "exports"}`,
    `${formatCount(groups.length)} ${groups.length === 1 ? "group" : "groups"}`,
    ...(exportsService.project.isLoading ? ["Refreshing"] : []),
  ]);

  return (
    <EditorLayout
      toolbar={
        <EditorToolbar
          subtitle={project?.root ? <EditorToolbarLocation location={{ path: project.root }} /> : null}
          actions={
            <>
              <ExportsSaveAction isDisabled={isBusy} />

              <EditorIconAction
                label={"Refresh exports"}
                description={"Refresh exports"}
                icon={<RefreshIcon />}
                isDisabled={isBusy}
                onClick={onRefresh}
              />
            </>
          }
          onBack={onClose}
        />
      }
      banner={
        exportsService.project.error ? (
          <Alert severity={"error"}>Could not refresh exports: {exportsService.project.error.message}</Alert>
        ) : null
      }
    >
      <ExportsViewer declaration={selectedDeclaration} exportCount={declarations.length} onDeselect={onDeselect} />
    </EditorLayout>
  );
}
