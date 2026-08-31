import { Box, CircularProgress, Theme, Typography } from "@mui/material";
import { SystemStyleObject } from "@mui/system";
import { clamp } from "@mui/x-data-grid/internals";
import { useInjection } from "@wirestate/react";
import { MouseEvent, ReactElement, useCallback, useMemo, useState, WheelEvent } from "react";

import { EquipmentGridControls } from "@/applications/sprite-equipment-editor/components/sprite-view/EquipmentGridControls";
import { EquipmentGridDetails } from "@/applications/sprite-equipment-editor/components/sprite-view/EquipmentGridDetails";
import { EquipmentGridMoveOver } from "@/applications/sprite-equipment-editor/components/sprite-view/EquipmentGridMoveOver";
import { EquipmentGridZoom } from "@/applications/sprite-equipment-editor/components/sprite-view/EquipmentGridZoom";
import { EquipmentSpriteGrid } from "@/applications/sprite-equipment-editor/components/sprite-view/EquipmentSpriteGrid";
import { equipmentViewerConfig } from "@/applications/sprite-equipment-editor/configs/EquipmentViewerConfig";
import { GridMapper, SpriteEquipmentService } from "@/core/sprite-equipment";
import { IMAGE_CHECKERBOARD } from "@/core/ui/media/media.styles";
import { Nullable } from "@/lib/types/general";

export function EquipmentSpriteViewer(): ReactElement {
  const spriteEquipmentService: SpriteEquipmentService = useInjection(SpriteEquipmentService);

  const [holdingOrigin, setHoldingOrigin] = useState<Nullable<[number, number]>>(null);
  const [zoomValue, setZoomValue] = useState(1);
  const [zoomOriginX, setZoomOriginX] = useState(0);
  const [zoomOriginY, setZoomOriginY] = useState(0);

  const [selectedCell, setSelectedCell] = useState<Nullable<[number, number]>>(null);
  const [moveOverCell, setMoveOverCell] = useState<Nullable<[number, number]>>(null);

  const gridMapper: Nullable<GridMapper> = useMemo(() => {
    if (!spriteEquipmentService.spriteImage.value) {
      return null;
    }

    return new GridMapper(
      spriteEquipmentService.spriteImage.value.image.width,
      spriteEquipmentService.spriteImage.value.image.height,
      spriteEquipmentService.gridSize,
      spriteEquipmentService.spriteImage.value.descriptors
    );
  }, [spriteEquipmentService.spriteImage.value, spriteEquipmentService.gridSize]);

  const sx: SystemStyleObject<Theme> = useMemo(
    () => ({
      ...IMAGE_CHECKERBOARD,
      backgroundColor: "#353535",
      userSelect: "none",
      transform: `scale(${zoomValue}) translate(${zoomOriginX}px, ${zoomOriginY}px)`,
    }),
    [zoomValue, zoomOriginX, zoomOriginY]
  );

  const onSelectCell = useCallback((row: number, column: number) => {
    setSelectedCell([row, column]);
  }, []);

  const onCloseDetails = useCallback(() => {
    setSelectedCell(null);
  }, []);

  const onMoveOverCell = useCallback((row: number, column: number) => {
    setMoveOverCell((it) => {
      return it && it[0] === row && it[1] === column ? it : [row, column];
    });
  }, []);

  const onZoomUp = useCallback(() => {
    setZoomValue((it) => clamp(it + 0.1, equipmentViewerConfig.ZOOM_IN_MIN, equipmentViewerConfig.ZOOM_IN_MAX));
  }, []);

  const onZoomDown = useCallback(() => {
    setZoomValue((it) => clamp(it - 0.1, equipmentViewerConfig.ZOOM_IN_MIN, equipmentViewerConfig.ZOOM_IN_MAX));
  }, []);

  const onWheel = useCallback((event: WheelEvent<HTMLDivElement>) => {
    if (event.shiftKey) {
      setZoomOriginY((it) =>
        clamp(
          event.deltaY > 0 ? it - 30 : it + 30,
          equipmentViewerConfig.ZOOM_OFFSET_MIN,
          equipmentViewerConfig.ZOOM_OFFSET_MAX
        )
      );
    } else if (event.ctrlKey) {
      setZoomOriginX((it) =>
        clamp(
          event.deltaY > 0 ? it - 30 : it + 30,
          equipmentViewerConfig.ZOOM_OFFSET_MIN,
          equipmentViewerConfig.ZOOM_OFFSET_MAX
        )
      );
    } else {
      setZoomValue((it) =>
        clamp(
          event.deltaY > 0 ? it - 0.1 : it + 0.1,
          equipmentViewerConfig.ZOOM_IN_MIN,
          equipmentViewerConfig.ZOOM_IN_MAX
        )
      );
    }
  }, []);

  const onMouseDown = useCallback((event: MouseEvent<HTMLDivElement>) => {
    setHoldingOrigin([event.pageX, event.pageY]);
  }, []);

  const onMouseUp = useCallback(() => {
    setHoldingOrigin(null);
  }, []);

  const onMouseLeave = useCallback(() => {
    setHoldingOrigin(null);
  }, []);

  const onContextMenu = useCallback((event: MouseEvent<HTMLDivElement>) => {
    event.preventDefault();
  }, []);

  const onMouseMove = useCallback(
    (event: MouseEvent<HTMLDivElement>) => {
      if (holdingOrigin) {
        const [x, y] = holdingOrigin;

        setZoomOriginX((it) =>
          clamp(
            it + (event.pageX - x) / 2,
            equipmentViewerConfig.ZOOM_OFFSET_MIN,
            equipmentViewerConfig.ZOOM_OFFSET_MAX
          )
        );
        setZoomOriginY((it) =>
          clamp(
            it + (event.pageY - y) / 2,
            equipmentViewerConfig.ZOOM_OFFSET_MIN,
            equipmentViewerConfig.ZOOM_OFFSET_MAX
          )
        );
        setHoldingOrigin([event.pageX, event.pageY]);
      }
    },
    [holdingOrigin]
  );

  return (
    <Box sx={{ width: "100%", height: "100%", position: "relative", overflow: "hidden" }}>
      <Box
        sx={{
          position: "absolute",
          display: "flex",
          justifyContent: "center",
          alignItems: "center",
          left: 0,
          top: 0,
          width: "100%",
          height: "100%",
        }}
      >
        {spriteEquipmentService.spriteImage.value ? (
          <Box
            className={"sprite-preview"}
            onWheel={onWheel}
            onMouseDown={onMouseDown}
            onMouseUp={onMouseUp}
            onMouseLeave={onMouseLeave}
            onContextMenu={onContextMenu}
            onMouseMove={onMouseMove}
            sx={[
              {
                position: "relative",
                width: spriteEquipmentService.spriteImage.value.image.width,
                minWidth: spriteEquipmentService.spriteImage.value.image.width,
                height: "auto",
                left: 0,
                top: 0,
              },
              sx,
            ]}
          >
            <img
              src={spriteEquipmentService.spriteImage.value.image.src}
              width={"100%"}
              height={"100%"}
              draggable={false}
            />

            {gridMapper ? (
              <EquipmentSpriteGrid
                selectedCell={selectedCell}
                isGridVisible={spriteEquipmentService.isGridVisible}
                gridMapper={gridMapper}
                onCellSelected={onSelectCell}
                onCellMovedOver={onMoveOverCell}
              />
            ) : null}
          </Box>
        ) : spriteEquipmentService.spriteImage.isLoading ? (
          <CircularProgress size={28} />
        ) : (
          <Typography variant={"body2"} color={"text.secondary"}>
            No sprite open
          </Typography>
        )}

        {selectedCell && gridMapper ? (
          <EquipmentGridDetails cell={selectedCell} gridMapper={gridMapper} onClose={onCloseDetails} />
        ) : null}

        {moveOverCell ? <EquipmentGridMoveOver cell={moveOverCell} /> : null}

        <EquipmentGridControls
          gridSize={spriteEquipmentService.gridSize}
          isGridVisible={spriteEquipmentService.isGridVisible}
          onSetGridSize={spriteEquipmentService.setGridSize}
          onSetGridVisibility={spriteEquipmentService.setGridVisibility}
        />

        <EquipmentGridZoom zoom={zoomValue} onZoomDown={onZoomDown} onZoomUp={onZoomUp} />
      </Box>
    </Box>
  );
}
