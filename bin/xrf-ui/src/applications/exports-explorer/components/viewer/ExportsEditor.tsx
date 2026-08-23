import { default as ListAltIcon } from "@mui/icons-material/ListAlt";
import { default as RefreshIcon } from "@mui/icons-material/Refresh";
import { Alert, Box, IconButton, Tooltip } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useMemo, useState } from "react";

import { groupExports, IExportGroup } from "@/applications/exports-explorer/components/viewer/exports/exports-groups";
import { ExportsMenu } from "@/applications/exports-explorer/components/viewer/exports/ExportsMenu";
import { ExportsViewer } from "@/applications/exports-explorer/components/viewer/exports/ExportsViewer";
import { ExportsService } from "@/applications/exports-explorer/services/exports";
import { ExportDescriptor, ExportsProject } from "@/core/bindings/types/xrf-export";
import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { useEditorBusy } from "@/core/shell/EditorBusyContext";
import { useEditorStatus } from "@/core/shell/EditorStatusContext";
import { useEditorPanels } from "@/core/shell/panel/context";
import { Nullable } from "@/lib/types/general";

export function ExportsEditor(): ReactElement {
  const exportsService: ExportsService = useInjection(ExportsService);

  const [selectedName, setSelectedName] = useState<Nullable<string>>(null);
  const [isClosing, setClosing] = useState<boolean>(false);
  const [closeError, setCloseError] = useState<Nullable<string>>(null);

  const project: Nullable<ExportsProject> = exportsService.project.value;
  const declarations: Array<ExportDescriptor> = useMemo(() => project?.declarations ?? [], [project?.declarations]);
  const groups: Array<IExportGroup> = useMemo(() => groupExports(declarations), [declarations]);
  const selectedDeclaration: Nullable<ExportDescriptor> =
    declarations.find((declaration: ExportDescriptor) => declaration.name === selectedName) ?? null;
  const isBusy: boolean = exportsService.project.isLoading || isClosing;

  const onSelect = useCallback((name: string): void => setSelectedName(name), []);

  const onRefresh = useCallback((): void => {
    setCloseError(null);
    void exportsService.refreshExportsProject();
  }, [exportsService]);

  const onClose = useCallback(async (): Promise<void> => {
    setClosing(true);
    setCloseError(null);

    try {
      // Closing does not navigate: the application shows its own picker again once nothing is open.
      await exportsService.closeExportsProject();
    } catch (error: unknown) {
      setCloseError(error instanceof Error ? error.message : String(error));
    } finally {
      setClosing(false);
    }
  }, [exportsService]);

  useEditorPanels(
    () => [
      {
        icon: <ListAltIcon />,
        id: "declarations",
        isOpenByDefault: true,
        label: "Declarations",
        render: () => <ExportsMenu declarations={declarations} selectedName={selectedName} onSelect={onSelect} />,
        side: "left",
      },
    ],
    [declarations, onSelect, selectedName]
  );

  useEffect(() => {
    if (selectedName && !selectedDeclaration) {
      setSelectedName(null);
    }
  }, [selectedDeclaration, selectedName]);

  useEditorBusy(isBusy);
  useEditorStatus([
    `${declarations.length} exports`,
    `${groups.length} groups`,
    ...(exportsService.project.isLoading ? ["Refreshing"] : []),
  ]);

  return (
    <EditorLayout
      toolbar={
        <EditorToolbar
          subtitle={
            project?.root ? (
              <Tooltip title={project.root}>
                <Box component={"span"} className={"monospace"}>
                  {project.root}
                </Box>
              </Tooltip>
            ) : null
          }
          actions={
            <Tooltip describeChild title={"Refresh exports"}>
              <span>
                <IconButton color={"inherit"} aria-label={"Refresh exports"} disabled={isBusy} onClick={onRefresh}>
                  <RefreshIcon fontSize={"small"} />
                </IconButton>
              </span>
            </Tooltip>
          }
          onBack={() => void onClose()}
        />
      }
      banner={
        <>
          {exportsService.project.error ? (
            <Alert severity={"error"}>Could not refresh exports: {exportsService.project.error.message}</Alert>
          ) : null}

          {closeError ? (
            <Alert severity={"error"} onClose={() => setCloseError(null)}>
              Could not close exports: {closeError}
            </Alert>
          ) : null}
        </>
      }
    >
      <ExportsViewer declaration={selectedDeclaration} exportCount={declarations.length} />
    </EditorLayout>
  );
}
