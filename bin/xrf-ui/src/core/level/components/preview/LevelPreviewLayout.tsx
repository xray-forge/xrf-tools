import { default as InfoOutlinedIcon } from "@mui/icons-material/InfoOutlined";
import { default as LayersIcon } from "@mui/icons-material/Layers";
import { default as SpeedIcon } from "@mui/icons-material/Speed";
import { default as WarningIcon } from "@mui/icons-material/WarningAmber";
import { useInjection } from "@wirestate/react";
import { ERendererRenderScale, IRendererFeatureSettings } from "@xrf/renderer";
import { Nullable } from "@xrf/types";
import { ReactElement, ReactNode, useCallback, useMemo } from "react";

import { LevelHeaderPanel } from "@/core/level/components/panels/LevelHeaderPanel";
import { LevelProblemsPanel } from "@/core/level/components/panels/LevelProblemsPanel";
import { LevelStreamPanel } from "@/core/level/components/panels/LevelStreamPanel";
import { LevelSurfacesPanel } from "@/core/level/components/panels/LevelSurfacesPanel";
import { LevelCameraAction } from "@/core/level/components/preview/LevelCameraAction";
import { LevelGoToAction } from "@/core/level/components/preview/LevelGoToAction";
import { LevelPreviewActivity } from "@/core/level/components/preview/LevelPreviewActivity";
import { LevelPreviewCoordinates } from "@/core/level/components/preview/LevelPreviewCoordinates";
import { LevelPreviewCover } from "@/core/level/components/preview/LevelPreviewCover";
import { LevelPreviewEmpty } from "@/core/level/components/preview/LevelPreviewEmpty";
import { LevelPreviewMetrics } from "@/core/level/components/preview/LevelPreviewMetrics";
import { LevelPreviewToolbar } from "@/core/level/components/preview/LevelPreviewToolbar";
import { ILevelPreviewViewportProps, LevelPreviewViewport } from "@/core/level/components/preview/LevelPreviewViewport";
import { ILevelCamera } from "@/core/level/lib/camera/level-camera";
import { ILevelGoTo } from "@/core/level/lib/camera/level-camera-goto";
import { ILevelFeatureOptions, TLevelFeatureView, toLevelFeatureView } from "@/core/level/lib/features";
import { LevelLoadService, LevelViewportService, LevelViewService } from "@/core/level/services";
import { SettingsService } from "@/core/settings/services/settings";
import { EditorFileHeader } from "@/core/shell/editor/EditorFileHeader";
import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { IEditorPanel, useEditorPanels } from "@/core/shell/editor-shell";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelPreviewLayoutProps extends BaseComponentProps {
  /** What the open level is called. Its presence is what draws the file header over the viewport. */
  name?: Nullable<string>;
  subtitle?: ReactNode;
  /** Whether the level itself is being opened, which is a different wait from streaming its sectors. */
  isLoading?: boolean;
  error?: string;
  /** Draws the viewport, for a surface with no webgl context to give one - a test, or a headless render. */
  renderViewport?: (props: ILevelPreviewViewportProps) => ReactNode;
  onRetry?: () => void;
  onBack?: () => void;
  onDeselect?: Nullable<() => void>;
}

/**
 * The level preview chrome: toolbar, viewport, panel stripe and the two waits a level has.
 */
export function LevelPreviewLayout({
  "data-testid": dataTestId = "level-preview-layout",
  id = "level-preview-layout",
  className,
  name = null,
  subtitle,
  isLoading = false,
  error,
  renderViewport,
  onRetry,
  onBack,
  onDeselect = null,
}: ILevelPreviewLayoutProps): ReactElement {
  const loadService: LevelLoadService = useInjection(LevelLoadService);
  const viewService: LevelViewService = useInjection(LevelViewService);
  const settingsService: SettingsService = useInjection(SettingsService);
  const viewportService: LevelViewportService = useInjection(LevelViewportService);

  const isOpen: boolean = Boolean(name);
  const settings: IRendererFeatureSettings = settingsService.rendererFeatures;
  const features: ILevelFeatureOptions = viewService.features;

  // Stable between changes of their own, so the toolbar redraws for a toggle and for nothing else.
  const featureView: TLevelFeatureView = useMemo(() => toLevelFeatureView(settings, features), [settings, features]);

  const readCamera = useCallback((): Nullable<ILevelCamera> => viewportService.camera, [viewportService]);
  const onGoTo = useCallback((goTo: ILevelGoTo) => viewService.requestGoTo(goTo), [viewService]);

  const actions: ReactElement = useMemo(
    () => (
      <>
        <LevelCameraAction camera={viewService.camera} onChange={viewService.setCamera} />
        <LevelGoToAction isDisabled={!isOpen} readCamera={readCamera} onGoTo={onGoTo} />
      </>
    ),
    [viewService.camera, viewService.setCamera, isOpen, readCamera, onGoTo]
  );

  const onChangeScale = useCallback(
    (scale: ERendererRenderScale) => settingsService.setRendererOverrides({ upscaling: { scale } }),
    [settingsService]
  );

  useEditorPanels(
    (): Array<IEditorPanel> => [
      {
        icon: <InfoOutlinedIcon />,
        id: "level",
        isOpenByDefault: true,
        label: "Level",
        render: () => <LevelHeaderPanel />,
      },
      {
        icon: <SpeedIcon />,
        id: "streaming",
        label: "Streaming",
        render: () => <LevelStreamPanel />,
      },
      {
        icon: <LayersIcon />,
        id: "surfaces",
        label: "Surfaces",
        render: () => <LevelSurfacesPanel />,
      },
      {
        icon: <WarningIcon />,
        id: "problems",
        label: "Problems",
        render: () => <LevelProblemsPanel />,
      },
    ],
    []
  );

  return (
    <EditorLayout
      toolbar={
        <LevelPreviewToolbar
          subtitle={subtitle}
          options={viewService.options}
          lighting={viewService.lighting}
          sun={loadService.level.value?.selected.value.sun ?? null}
          lod={viewService.lod}
          features={features}
          featureView={featureView}
          settings={settings}
          actions={actions}
          onChangeOptions={viewService.setOptions}
          onChangeLighting={viewService.setLighting}
          onChangeLod={viewService.setLod}
          onChangeFeatures={viewService.setFeatures}
          onChangeScale={onChangeScale}
          onBack={onBack}
        />
      }
    >
      <div className={"flex min-h-0 min-w-0 grow flex-col"}>
        {name ? (
          <EditorFileHeader
            data-testid={"level-file-header"}
            name={name}
            closeLabel={"Close level"}
            closeDescription={"Clear the selection and close this level"}
            onClose={onDeselect ?? undefined}
          />
        ) : null}

        <div
          data-testid={dataTestId}
          id={id}
          className={cn("relative flex min-h-0 min-w-0 flex-1 overflow-hidden", className)}
        >
          {renderViewport ? renderViewport({}) : <LevelPreviewViewport />}

          {isOpen && viewService.options.isStatsVisible ? (
            <>
              <LevelPreviewMetrics />
              <LevelPreviewCoordinates />
            </>
          ) : null}

          {!isOpen && !isLoading ? <LevelPreviewEmpty error={error} onRetry={onRetry} /> : null}

          {isOpen || isLoading ? <LevelPreviewCover isLoading={isLoading} /> : null}

          <LevelPreviewActivity isOpen={isOpen} isLoading={isLoading} />
        </div>
      </div>
    </EditorLayout>
  );
}
