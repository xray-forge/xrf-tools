import { default as InfoOutlinedIcon } from "@mui/icons-material/InfoOutlined";
import { default as Inventory2Icon } from "@mui/icons-material/Inventory2";
import { default as LayersIcon } from "@mui/icons-material/Layers";
import { default as SpeedIcon } from "@mui/icons-material/Speed";
import { default as WarningIcon } from "@mui/icons-material/WarningAmber";
import { default as WbCloudyIcon } from "@mui/icons-material/WbCloudy";
import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement, ReactNode, useCallback, useMemo } from "react";

import { LevelWeatherTexture } from "@/core/ipc/types/xrf-app";
import { ThunderboltCollection } from "@/core/ipc/types/xrf-environment";
import { LevelHeaderPanel } from "@/core/level/components/panels/LevelHeaderPanel";
import { LevelProblemsPanel } from "@/core/level/components/panels/LevelProblemsPanel";
import { LevelRendererPanel } from "@/core/level/components/panels/LevelRendererPanel";
import { LevelSpawnPanel } from "@/core/level/components/panels/LevelSpawnPanel";
import { LevelSurfacesPanel } from "@/core/level/components/panels/LevelSurfacesPanel";
import { LevelWeatherPanel } from "@/core/level/components/panels/LevelWeatherPanel";
import { LevelCameraAction } from "@/core/level/components/preview/LevelCameraAction";
import { LevelGoToAction } from "@/core/level/components/preview/LevelGoToAction";
import { LevelPreviewCoordinates } from "@/core/level/components/preview/LevelPreviewCoordinates";
import { LevelPreviewCover } from "@/core/level/components/preview/LevelPreviewCover";
import { LevelPreviewEmpty } from "@/core/level/components/preview/LevelPreviewEmpty";
import { LevelPreviewMetrics } from "@/core/level/components/preview/LevelPreviewMetrics";
import { LevelPreviewPick } from "@/core/level/components/preview/LevelPreviewPick";
import { LevelPreviewToolbar } from "@/core/level/components/preview/LevelPreviewToolbar";
import { ILevelPreviewViewportProps, LevelPreviewViewport } from "@/core/level/components/preview/LevelPreviewViewport";
import { ILevelCamera } from "@/core/level/lib/camera/level-camera";
import { ILevelGoTo } from "@/core/level/lib/camera/level-camera-goto";
import { ILevelFeatureOptions, TLevelFeatureView, toLevelFeatureView } from "@/core/level/lib/features";
import { ELevelPanelId } from "@/core/level/lib/panels/level-panel-id";
import {
  LevelLoadService,
  LevelRenderService,
  LevelViewportService,
  LevelViewService,
  LevelWeatherService,
} from "@/core/level/services";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/render-feature-settings";
import { SettingsService } from "@/core/settings/services/settings";
import { EditorFileHeader } from "@/core/shell/editor/EditorFileHeader";
import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { IEditorPanel, useEditorPanels } from "@/core/shell/editor-shell";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

/** What the sky and clouds popovers offer before the level's weather is read. */
const EMPTY_TEXTURES: ReadonlyArray<LevelWeatherTexture> = [];

/** What the thunder's popover offers before it is read. */
const EMPTY_COLLECTIONS: ReadonlyArray<ThunderboltCollection> = [];

/** Lens flares while the weather is not read yet. */
const EMPTY_SUNS: ReadonlyArray<string> = [];

interface ILevelPreviewLayoutProps extends BaseComponentProps {
  /** What the open level is called, which heads the viewport. Its presence is what offers go-to, stats and picks. */
  name?: Nullable<string>;
  /**
   * What the level being opened is called, which heads the viewport until it is open: the header comes in with the open
   * otherwise, and the viewport shrinks by it, which sizes every target the renderer draws into again.
   */
  pending?: Nullable<string>;
  subtitle?: ReactNode;
  /** Whether the level itself is being opened, which is a different wait from streaming its sectors. */
  isLoading?: boolean;
  error?: string;
  /** Draws the viewport, for a surface with no GPU to give one - a test, or a headless render. */
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
  pending = null,
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
  const renderService: LevelRenderService = useInjection(LevelRenderService);
  const weatherService: LevelWeatherService = useInjection(LevelWeatherService);

  const isOpen: boolean = Boolean(name);
  const heading: Nullable<string> = name ?? pending;
  const settings: IRenderFeatureSettings = settingsService.rendererFeatures;
  const features: ILevelFeatureOptions = viewService.features;

  // Stable between changes of their own, so the toolbar redraws for a toggle and for nothing else.
  const featureView: TLevelFeatureView = useMemo(() => toLevelFeatureView(settings, features), [settings, features]);

  const readCamera = useCallback((): Nullable<ILevelCamera> => viewportService.camera, [viewportService]);
  const onGoTo = useCallback((goTo: ILevelGoTo) => renderService.goTo(goTo), [renderService]);

  const actions: ReactElement = useMemo(
    () => (
      <>
        <LevelCameraAction camera={viewService.camera} onChange={viewService.setCamera} />
        <LevelGoToAction isDisabled={!isOpen} readCamera={readCamera} onGoTo={onGoTo} />
      </>
    ),
    [viewService.camera, viewService.setCamera, isOpen, readCamera, onGoTo]
  );

  useEditorPanels(
    (): Array<IEditorPanel> => [
      {
        icon: <WbCloudyIcon />,
        id: ELevelPanelId.WEATHER,
        isOpenByDefault: true,
        label: "Weather",
        render: () => <LevelWeatherPanel />,
        side: "left",
      },
      {
        icon: <Inventory2Icon />,
        id: ELevelPanelId.SPAWN,
        label: "Spawn",
        render: () => <LevelSpawnPanel />,
        side: "left",
      },
      {
        icon: <InfoOutlinedIcon />,
        id: ELevelPanelId.LEVEL,
        isOpenByDefault: true,
        label: "Level",
        render: () => <LevelHeaderPanel />,
      },
      {
        icon: <SpeedIcon />,
        id: ELevelPanelId.RENDERER,
        label: "Renderer",
        render: () => <LevelRendererPanel />,
      },
      {
        icon: <LayersIcon />,
        id: ELevelPanelId.SURFACES,
        label: "Surfaces",
        render: () => <LevelSurfacesPanel />,
        side: "left",
      },
      {
        icon: <WarningIcon />,
        id: ELevelPanelId.PROBLEMS,
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
          hemiStrength={viewService.hemiStrength}
          manual={weatherService.shown}
          skies={weatherService.description?.skies ?? EMPTY_TEXTURES}
          clouds={weatherService.description?.clouds ?? EMPTY_TEXTURES}
          collections={weatherService.description?.thunderboltCollections ?? EMPTY_COLLECTIONS}
          sun={loadService.level.value?.selected.value.sun ?? null}
          drawnSun={renderService.applied?.sun ?? null}
          suns={weatherService.description?.suns ?? EMPTY_SUNS}
          sunShafts={weatherService.sunShafts}
          lod={viewService.lod}
          features={features}
          featureView={featureView}
          settings={settings}
          isGpuTimed={settingsService.isGpuTimed}
          shading={viewService.shading}
          actions={actions}
          onChangeOptions={viewService.setOptions}
          onChangeHemiStrength={viewService.setHemiStrength}
          onChangeSunShafts={weatherService.setSunShafts}
          onEditManual={weatherService.editManual}
          onChangeLod={viewService.setLod}
          onChangeFeatures={viewService.setFeatures}
          onChangeGpuTimed={settingsService.setGpuTimed}
          onChangeShading={viewService.setShading}
          onBack={onBack}
        />
      }
    >
      <div className={"flex min-h-0 min-w-0 grow flex-col"}>
        {heading ? (
          <EditorFileHeader
            data-testid={"level-file-header"}
            name={heading}
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

          {isOpen ? <LevelPreviewPick /> : null}

          {!isOpen && !isLoading ? <LevelPreviewEmpty error={error} onRetry={onRetry} /> : null}

          {isOpen || isLoading ? <LevelPreviewCover isLoading={isLoading} /> : null}
        </div>
      </div>
    </EditorLayout>
  );
}
