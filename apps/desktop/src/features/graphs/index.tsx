import type { ReactElement } from "react";
import { useCallback, useLayoutEffect, useEffect, useMemo, useRef, useState, type PointerEvent as ReactPointerEvent, type WheelEvent } from "react";
import { GitGraph, Move, RefreshCcwDot, RotateCcw, ZoomIn, ZoomOut } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Empty } from "@/components/ui/empty";
import { Typography } from "@/components/ui/typography";
import { useAppControllerContext } from "@/providers/AppControllerProvider";

type NodePosition = {
  id: string;
  label: string;
  nodeType: string;
  x: number;
  y: number;
};

type EdgePosition = {
  id: string;
  source: string;
  target: string;
  edgeType: string;
};

type NativePointerEvent = globalThis.PointerEvent;

type PointEvent = {
  clientX: number;
  clientY: number;
};

type GraphViewBox = {
  height: number;
  width: number;
  x: number;
  y: number;
};

type DragState = {
  id: string;
  offsetX: number;
  offsetY: number;
};

type GraphTooltip = {
  detail: string;
  kind: "edge" | "node";
  title: string;
  x: number;
  y: number;
};

type PanState = {
  startX: number;
  startY: number;
  viewBox: GraphViewBox;
};

type GraphLayout = "circle" | "star" | "neuron" | "random";

const DEFAULT_VIEW_BOX: GraphViewBox = {
  height: 560,
  width: 960,
  x: 0,
  y: 0,
};

const MIN_ZOOM = 80;
const MAX_ZOOM = 2400;

function toNodePosition(node: { id: string; name: string; nodeType: string }): NodePosition {
  return positionNodesCircle([node])[0];
}

export function GraphsPage(): ReactElement {
  const controller = useAppControllerContext();
  const [isGraphOpen, setIsGraphOpen] = useState(false);
  const [layout, setLayout] = useState<GraphLayout>("circle");
  const edgeCount = controller.graphEdges.length;
  const nodeCount = controller.graphNodes.length;
  const hasGraph = nodeCount > 0 || edgeCount > 0;
  const rootNode = controller.graphNodes.find((node) => node.nodeType === "Repository") ?? controller.graphNodes[0];
  const rootPreviewNode = rootNode ? toNodePosition(rootNode) : undefined;

  useLayoutEffect(() => {
    if (!isGraphOpen) return;
    const previousOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    function handleKeyDown(event: KeyboardEvent): void {
      if (event.key === "Escape") setIsGraphOpen(false);
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      document.body.style.overflow = previousOverflow;
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [isGraphOpen]);

  return (
    <section className="grid gap-6">
      <div className="grid gap-5 md:grid-cols-2 xl:grid-cols-4">
        <MetricCard label="Nodes" value={String(nodeCount)} />
        <MetricCard label="Edges" value={String(edgeCount)} />
        <MetricCard label="Files" value={String(controller.scan?.files ?? 0)} />
        <MetricCard label="Status" value={controller.status} />
      </div>

      <Card>
        <CardHeader className="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
          <div>
            <CardTitle>Repository Graph</CardTitle>
            <CardDescription className="max-w-[calc(100%-50px)]">Graph root preview is shown on this page. Open the full graph workspace to pan, zoom, drag nodes, and inspect relationships.</CardDescription>
          </div>
          <div className="flex flex-col gap-2 justify-end">
            <Button variant="secondary" size="sm" className="w-25" type="button" onClick={controller.onAnalyze} disabled={controller.status === "working"}>
              Rebuild graph
            </Button>
            <Button size="sm" className="w-25" type="button" onClick={() => setIsGraphOpen(true)} disabled={!hasGraph}>
              Open full graph
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          {hasGraph ? (
            <div className="grid gap-5 xl:grid-cols-[minmax(0,1fr)_320px]">
              <GraphRootPreview node={rootPreviewNode} onOpen={() => setIsGraphOpen(true)} />
              <GraphDetails edgeCount={edgeCount} nodeCount={nodeCount} nodeTypes={getNodeTypeCounts(controller.graphNodes)} />
            </div>
          ) : (
            <Empty>
              <GitGraph size={24} />
              <div>
                <Typography variant="h3">No graph available yet.</Typography>
                <Typography variant="muted">Run Analyze in Scanner to build the repository graph.</Typography>
              </div>
            </Empty>
          )}
        </CardContent>
      </Card>
      {isGraphOpen && (
        <div className="fixed inset-0 z-[80] grid grid-rows-[auto_minmax(0,1fr)] bg-background/95 backdrop-blur-xl">
          <header className="grid gap-3 border-b border-border bg-card/90 px-5 py-4 sm:px-6">
            <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
              <div>
                <h2 className="m-0 text-xl font-semibold leading-7 text-foreground">Full Graph Workspace</h2>
                <p className="m-0 text-sm leading-6 text-muted-foreground">Pan the canvas, drag nodes, zoom with the wheel, and hover nodes or edges for details.</p>
              </div>
              <div className="flex flex-wrap gap-2">
                <MetricPill label="Nodes" value={String(nodeCount)} />
                <MetricPill label="Edges" value={String(edgeCount)} />
                <Button variant="secondary" className="min-h-9" type="button" onClick={() => setIsGraphOpen(false)}>
                  Close
                </Button>
              </div>
            </div>
          </header>
          <main className="min-h-0 overflow-hidden p-4 sm:p-6">
            <GraphCanvas className="h-[calc(100%-100px)]" edges={controller.graphEdges} nodes={controller.graphNodes} layout={layout} onLayoutChange={setLayout} />
          </main>
        </div>
      )}
    </section>
  );
}

function MetricCard(props: { label: string; value: string }): ReactElement {
  return (
    <Card compact>
      <CardDescription>{props.label}</CardDescription>
      <Typography variant="h2">{props.value}</Typography>
    </Card>
  );
}

function MetricPill(props: { label: string; value: string }): ReactElement {
  return (
    <div className="grid min-h-9 place-items-center gap-0.5 rounded-none border bg-card px-3 text-xs leading-5">
      <span className="text-muted-foreground">{props.label}</span>
      <strong className="text-foreground">{props.value}</strong>
    </div>
  );
}

function GraphRootPreview(props: {
  node?: NodePosition;
  onOpen: () => void;
}): ReactElement {
  if (!props.node) {
    return (
      <Empty>
        <GitGraph size={24} />
        <div>
          <Typography variant="h3">No graph root found.</Typography>
          <Typography variant="muted">Run Analyze again to rebuild the repository graph.</Typography>
        </div>
      </Empty>
    );
  }

  return (
    <div className="relative grid min-h-[340px] place-items-center overflow-hidden rounded-none border bg-muted/40">
      <div className="absolute inset-0 bg-[radial-gradient(circle_at_center,hsl(var(--primary)/0.16),transparent_34%)]" />
      <div className="relative z-10 grid place-items-center gap-4 text-center">
        <div className="grid h-28 w-28 place-items-center rounded-none border bg-primary text-primary-foreground shadow-xl">
          <GitGraph size={34} />
        </div>
        <div className="grid max-w-[520px] gap-1">
          <strong className="break-all text-lg font-semibold leading-7 text-foreground">{props.node.label}</strong>
          <span className="text-sm leading-6 text-muted-foreground">{props.node.nodeType}</span>
          <p className="m-0 text-xs leading-5 text-muted-foreground">This page only shows the graph origin. Open the full workspace to interact with the complete graph.</p>
        </div>
        <Button type="button" onClick={props.onOpen}>Open full graph</Button>
      </div>
    </div>
  );
}

function GraphCanvas(props: {
  className?: string;
  edges: EdgePosition[];
  nodes: Array<{ id: string; name: string; nodeType: string }>;
  layout?: GraphLayout;
  onLayoutChange?: (layout: GraphLayout) => void;
}): ReactElement {
  const svgRef = useRef<SVGSVGElement | null>(null);
  const [viewBox, setViewBox] = useState<GraphViewBox>(() => createFitViewBox(applyLayout(props.nodes, props.layout ?? "circle"), DEFAULT_VIEW_BOX.width, DEFAULT_VIEW_BOX.height));
  const [draggedPositions, setDraggedPositions] = useState<Record<string, { x: number; y: number }>>({});
  const [dragNode, setDragNode] = useState<DragState | null>(null);
  const [hoveredNodeId, setHoveredNodeId] = useState<string | null>(null);
  const [panStart, setPanStart] = useState<PanState | null>(null);
  const [tooltip, setTooltip] = useState<GraphTooltip | null>(null);
  const dragNodeRef = useRef<DragState | null>(null);
  const panStartRef = useRef<PanState | null>(null);
  const [layout, setLayout] = useState<GraphLayout>(props.layout ?? "circle");

  const basePositions = useMemo(() => applyLayout(props.nodes, layout), [props.nodes, layout]);

  const nodes = useMemo(() => {
    if (Object.keys(draggedPositions).length === 0) return basePositions;
    return basePositions.map((node) => {
      const dragged = draggedPositions[node.id];
      return dragged ? { ...node, x: dragged.x, y: dragged.y } : node;
    });
  }, [basePositions, draggedPositions]);

  const clientToSvgPoint = useCallback((event: PointEvent, viewBoxOverride: GraphViewBox = viewBox): { x: number; y: number } => {
    const svg = svgRef.current;
    if (!svg) return { x: DEFAULT_VIEW_BOX.width / 2, y: DEFAULT_VIEW_BOX.height / 2 };
    const rect = svg.getBoundingClientRect();
    return {
      x: ((event.clientX - rect.left) / rect.width) * viewBoxOverride.width + viewBoxOverride.x,
      y: ((event.clientY - rect.top) / rect.height) * viewBoxOverride.height + viewBoxOverride.y,
    };
  }, [viewBox]);

  const fitGraph = useCallback(() => {
    setViewBox(createFitViewBox(nodes, DEFAULT_VIEW_BOX.width, DEFAULT_VIEW_BOX.height));
  }, [nodes]);

  const resetGraph = useCallback(() => {
    setViewBox(DEFAULT_VIEW_BOX);
  }, []);

  const resetNodes = useCallback(() => {
    setDraggedPositions({});
    setDragNode(null);
    dragNodeRef.current = null;
  }, []);

  const zoomAt = useCallback((factor: number, anchor: { x: number; y: number }) => {
    setViewBox((current) => {
      const nextWidth = clamp(current.width * factor, MIN_ZOOM, MAX_ZOOM);
      const nextHeight = current.height * (nextWidth / current.width);
      const zoomX = anchor.x - ((anchor.x - current.x) * (nextWidth / current.width));
      const zoomY = anchor.y - ((anchor.y - current.y) * (nextHeight / current.height));
      return {
        height: nextHeight,
        width: nextWidth,
        x: zoomX,
        y: zoomY,
      };
    });
  }, []);

  const handleWheel = useCallback((event: WheelEvent<SVGSVGElement>) => {
    event.preventDefault();
    const point = clientToSvgPoint(event.nativeEvent);
    zoomAt(Math.exp(-event.deltaY * 0.001), point);
  }, [clientToSvgPoint, zoomAt]);

  const handleWindowPointerMove = useCallback((event: NativePointerEvent) => {
    const activePan = panStartRef.current;
    const activeDrag = dragNodeRef.current;
    if (activePan) {
      const point = clientToSvgPoint(event, activePan.viewBox);
      setViewBox({
        height: activePan.viewBox.height,
        width: activePan.viewBox.width,
        x: activePan.viewBox.x - (point.x - activePan.startX),
        y: activePan.viewBox.y - (point.y - activePan.startY),
      });
      return;
    }
    if (activeDrag) {
      const point = clientToSvgPoint(event);
      setDraggedPositions((current) => ({
        ...current,
        [activeDrag.id]: { x: point.x - activeDrag.offsetX, y: point.y - activeDrag.offsetY },
      }));
    }
  }, [clientToSvgPoint]);

  const stopWindowInteractionRef = useRef<() => void>(() => {});

  const stopWindowInteraction = useCallback(() => {
    dragNodeRef.current = null;
    panStartRef.current = null;
    setDragNode(null);
    setPanStart(null);
    window.removeEventListener("pointermove", handleWindowPointerMove);
    window.removeEventListener("pointerup", stopWindowInteractionRef.current);
    window.removeEventListener("pointercancel", stopWindowInteractionRef.current);
  }, [handleWindowPointerMove]);

  useLayoutEffect(() => {
    stopWindowInteractionRef.current = stopWindowInteraction;
  }, [stopWindowInteraction]);

  useEffect(() => {
    return () => {
      window.removeEventListener("pointermove", handleWindowPointerMove);
      window.removeEventListener("pointerup", stopWindowInteraction);
      window.removeEventListener("pointercancel", stopWindowInteraction);
    };
  }, [handleWindowPointerMove, stopWindowInteraction]);

  function startPan(event: PointEvent): void {
    const point = clientToSvgPoint(event);
    const nextPanStart = {
      startX: point.x,
      startY: point.y,
      viewBox,
    };
    panStartRef.current = nextPanStart;
    setPanStart(nextPanStart);
    window.addEventListener("pointermove", handleWindowPointerMove);
    window.addEventListener("pointerup", stopWindowInteraction, { once: true });
    window.addEventListener("pointercancel", stopWindowInteraction, { once: true });
  }

  function handleNodePointerDown(event: ReactPointerEvent<SVGGElement>, node: NodePosition): void {
    event.stopPropagation();
    if (event.button === 2) {
      event.preventDefault();
      startPan(event.nativeEvent);
      return;
    }
    const point = clientToSvgPoint(event.nativeEvent);
    setDragNode({
      id: node.id,
      offsetX: point.x - node.x,
      offsetY: point.y - node.y,
    });
    dragNodeRef.current = {
      id: node.id,
      offsetX: point.x - node.x,
      offsetY: point.y - node.y,
    };
    setHoveredNodeId(node.id);
    window.addEventListener("pointermove", handleWindowPointerMove);
    window.addEventListener("pointerup", stopWindowInteraction, { once: true });
    window.addEventListener("pointercancel", stopWindowInteraction, { once: true });
  }

  function handleNodePointerEnter(event: ReactPointerEvent<SVGGElement>, node: NodePosition): void {
    setHoveredNodeId(node.id);
    setTooltip({
      detail: `${node.nodeType} · ${node.id}`,
      kind: "node",
      title: node.label,
      x: event.clientX,
      y: event.clientY,
    });
  }

  function handleNodePointerMove(event: ReactPointerEvent<SVGGElement>): void {
    if (!tooltip) return;
    setTooltip({ ...tooltip, x: event.clientX, y: event.clientY });
  }

  function handleNodePointerLeave(): void {
    if (!dragNode) {
      setHoveredNodeId(null);
      setTooltip(null);
    }
  }

  function handleCanvasPointerDown(event: ReactPointerEvent<SVGSVGElement>): void {
    if (event.button !== 0 && event.button !== 2) return;
    event.preventDefault();
    startPan(event.nativeEvent);
  }

  function handleCanvasPointerMove(event: ReactPointerEvent<SVGSVGElement>): void {
    const point = clientToSvgPoint(event.nativeEvent);
    if (dragNode) {
      setDraggedPositions((current) => ({
        ...current,
        [dragNode.id]: { x: point.x - dragNode.offsetX, y: point.y - dragNode.offsetY },
      }));
      return;
    }
    if (panStart) {
      setViewBox({
        height: panStart.viewBox.height,
        width: panStart.viewBox.width,
        x: panStart.viewBox.x - (point.x - panStart.startX),
        y: panStart.viewBox.y - (point.y - panStart.startY),
      });
    }
  }

  function handleCanvasPointerUp(): void {
    stopWindowInteraction();
  }

  function handleEdgePointerEnter(event: ReactPointerEvent<SVGLineElement>, edge: EdgePosition): void {
    setTooltip({
      detail: `${edge.edgeType} · ${edge.source} → ${edge.target}`,
      kind: "edge",
      title: edge.id,
      x: event.clientX,
      y: event.clientY,
    });
  }

  function handleEdgePointerMove(event: ReactPointerEvent<SVGLineElement>): void {
    if (!tooltip) return;
    setTooltip({ ...tooltip, x: event.clientX, y: event.clientY });
  }

  function handleEdgePointerLeave(): void {
    setTooltip(null);
  }

  function zoomIn(): void {
    zoomAt(1.25, { x: viewBox.x + viewBox.width / 2, y: viewBox.y + viewBox.height / 2 });
  }

  function zoomOut(): void {
    zoomAt(0.8, { x: viewBox.x + viewBox.width / 2, y: viewBox.y + viewBox.height / 2 });
  }

  const connectedNodeIds = new Set(hoveredNodeId ? getConnectedNodeIds(props.edges, hoveredNodeId) : []);

  const changeLayout = useCallback((next: GraphLayout) => {
    setDraggedPositions({});
    setLayout(next);
  }, []);

  return (
    <div className="relative h-full select-none overflow-hidden rounded-none border bg-muted/40" onContextMenu={(event) => event.preventDefault()}>
      <div className="absolute right-3 top-3 z-10 flex gap-2 rounded-none border bg-card/90 p-1 backdrop-blur-xl">
        <Button size="icon" variant="secondary" type="button" onClick={zoomIn} aria-label="Zoom in">
          <ZoomOut size={16} />
        </Button>
        <Button size="icon" variant="secondary" type="button" onClick={zoomOut} aria-label="Zoom out">
          <ZoomIn size={16} />
        </Button>
        <Button size="icon" variant="secondary" type="button" onClick={fitGraph} aria-label="Fit graph">
          <Move size={16} />
        </Button>
        <Button size="icon" variant="secondary" type="button" onClick={resetGraph} aria-label="Reset graph">
          <RotateCcw size={16} />
        </Button>
        <Button size="icon" variant="secondary" type="button" onClick={resetNodes} aria-label="Reset node positions">
          <RefreshCcwDot size={16} />
        </Button>
      </div>
      <div className="absolute right-3 top-20 z-10 flex gap-1 rounded-none border bg-card/90 p-1 backdrop-blur-xl">
        <Button size="icon" variant="secondary" type="button" onClick={() => changeLayout("circle")} aria-label="Circle layout" className={layout === "circle" ? "ring-1 ring-primary" : ""}>
          <svg viewBox="0 0 24 24" className="size-4" fill="none" stroke="currentColor" strokeWidth="2"><circle cx="12" cy="12" r="9" /></svg>
        </Button>
        <Button size="icon" variant="secondary" type="button" onClick={() => changeLayout("star")} aria-label="Star layout" className={layout === "star" ? "ring-1 ring-primary" : ""}>
          <svg viewBox="0 0 24 24" className="size-4" fill="none" stroke="currentColor" strokeWidth="2"><path d="M12 2l3.09 6.26L22 9.27l-5 4.87L18.18 22 12 18.56 5.82 22 7 14.14l-5-4.87 6.91-1.01L12 2z" /></svg>
        </Button>
        <Button size="icon" variant="secondary" type="button" onClick={() => changeLayout("neuron")} aria-label="Neuron layout" className={layout === "neuron" ? "ring-1 ring-primary" : ""}>
          <svg viewBox="0 0 24 24" className="size-4" fill="none" stroke="currentColor" strokeWidth="2"><circle cx="12" cy="12" r="3"/><circle cx="5" cy="6" r="2"/><circle cx="19" cy="6" r="2"/><circle cx="5" cy="18" r="2"/><circle cx="19" cy="18" r="2"/><path d="M7.5 7.5L10.5 10.5M16.5 7.5L13.5 10.5M7.5 16.5L10.5 13.5M16.5 16.5L13.5 13.5"/></svg>
        </Button>
        <Button size="icon" variant="secondary" type="button" onClick={() => changeLayout("random")} aria-label="Random layout" className={layout === "random" ? "ring-1 ring-primary" : ""}>
          <svg viewBox="0 0 24 24" className="size-4" fill="none" stroke="currentColor" strokeWidth="2"><path d="M3 3l7.07 16.97 2.51-7.39 7.39-2.51L3 3zM13 13l6-6"/></svg>
        </Button>
      </div>
      <div className="absolute left-3 top-3 z-10 rounded-none border bg-card/90 px-3 py-2 text-xs leading-5 text-muted-foreground backdrop-blur-xl">
        Left drag node · Right drag canvas to pan · Wheel to zoom
      </div>
      <svg
        ref={svgRef}
        className={["h-full w-full touch-none", props.className].filter(Boolean).join(" ")}
        viewBox={`${viewBox.x} ${viewBox.y} ${viewBox.width} ${viewBox.height}`}
        onPointerDown={handleCanvasPointerDown}
        onPointerMove={handleCanvasPointerMove}
        onPointerUp={handleCanvasPointerUp}
        onWheel={handleWheel}
        role="img"
        aria-label="Repository graph"
        style={{ cursor: panStart ? "grabbing" : "grab" }}
      >
        <defs>
          <marker id="arrow" markerHeight="8" markerWidth="8" orient="auto" refX="10" refY="4" viewBox="0 0 10 8">
            <path className="fill-primary/70" d="M0 0 10 4 0 8Z" />
          </marker>
        </defs>
        <rect className="fill-transparent" height={viewBox.height} width={viewBox.width} x={viewBox.x} y={viewBox.y} />
        {props.edges.map((edge) => {
          const source = nodes.find((node) => node.id === edge.source);
          const target = nodes.find((node) => node.id === edge.target);
          if (!source || !target) return null;
          const isActive = hoveredNodeId === null || hoveredNodeId === edge.source || hoveredNodeId === edge.target || connectedNodeIds.has(edge.source) || connectedNodeIds.has(edge.target);
          return (
            <line
              className={isActive ? "stroke-primary/50" : "stroke-primary/15"}
              key={edge.id}
              markerEnd="url(#arrow)"
              onPointerEnter={(event) => handleEdgePointerEnter(event, edge)}
              onPointerLeave={handleEdgePointerLeave}
              onPointerMove={handleEdgePointerMove}
              strokeWidth={isActive ? 2 : 1}
              x1={source.x}
              x2={target.x}
              y1={source.y}
              y2={target.y}
            />
          );
        })}
        {nodes.map((node) => (
          <g
            key={node.id}
            transform={`translate(${node.x},${node.y})`}
            onPointerDown={(event) => handleNodePointerDown(event, node)}
            onPointerEnter={(event) => handleNodePointerEnter(event, node)}
            onPointerLeave={handleNodePointerLeave}
            onPointerMove={handleNodePointerMove}
            style={{ cursor: dragNode?.id === node.id ? "grabbing" : "grab" }}
          >
            <circle className={nodeClass(node.nodeType)} r={hoveredNodeId === node.id ? 22 : 18} />
            <circle className="fill-primary-foreground" r="5" />
            <text className="fill-foreground text-[11px] font-medium" dy="34" textAnchor="middle">{node.label}</text>
            <text className="fill-muted-foreground text-[9px]" dy="47" textAnchor="middle">{node.nodeType}</text>
          </g>
        ))}
      </svg>
      {tooltip && (
        <div
          className="pointer-events-none fixed z-50 max-w-fit z-50 rounded-none border bg-card/95 px-3 py-2 text-xs leading-5 text-foreground shadow-xl backdrop-blur-xl"
          style={{ left: tooltip.x + 14, top: tooltip.y + 14 }}
        >
          <strong>{tooltip.title}</strong>
          <div className="mt-1 text-muted-foreground">{tooltip.detail}</div>
        </div>
      )}
    </div>
  );
}

function GraphDetails(props: {
  edgeCount: number;
  nodeCount: number;
  nodeTypes: Array<{ count: number; label: string }>;
}): ReactElement {
  return (
    <div className="grid gap-3">
      <Card compact>
        <CardDescription>Node type distribution</CardDescription>
        <div className="mt-3 grid gap-2">
          {props.nodeTypes.map((nodeType) => (
            <div className="grid grid-cols-[minmax(0,1fr)_auto] gap-3 rounded-none bg-muted p-3" key={nodeType.label}>
              <span className="text-sm leading-6 text-muted-foreground">{nodeType.label}</span>
              <Badge>{nodeType.count}</Badge>
            </div>
          ))}
        </div>
      </Card>
      <Card compact>
        <CardDescription>Graph summary</CardDescription>
        <div className="mt-3 grid gap-2 text-sm leading-6 text-muted-foreground">
          <div className="grid grid-cols-[minmax(0,1fr)_auto] gap-3 rounded-none bg-muted p-3">
            <span>Total nodes</span>
            <strong className="text-foreground">{props.nodeCount}</strong>
          </div>
          <div className="grid grid-cols-[minmax(0,1fr)_auto] gap-3 rounded-none bg-muted p-3">
            <span>Total edges</span>
            <strong className="text-foreground">{props.edgeCount}</strong>
          </div>
        </div>
      </Card>
    </div>
  );
}

function createFitViewBox(nodes: NodePosition[], width: number, height: number): GraphViewBox {
  if (nodes.length === 0) return DEFAULT_VIEW_BOX;
  const padding = 140;
  const minX = Math.min(...nodes.map((node) => node.x)) - padding;
  const maxX = Math.max(...nodes.map((node) => node.x)) + padding;
  const minY = Math.min(...nodes.map((node) => node.y)) - padding;
  const maxY = Math.max(...nodes.map((node) => node.y)) + padding;
  const graphWidth = Math.max(maxX - minX, 1);
  const graphHeight = Math.max(maxY - minY, 1);
  const aspect = width / height;
  let nextWidth = graphWidth;
  let nextHeight = graphHeight;
  if (nextWidth / nextHeight > aspect) {
    nextHeight = nextWidth / aspect;
  } else {
    nextWidth = nextHeight * aspect;
  }
  return {
    height: nextHeight,
    width: nextWidth,
    x: minX - (nextWidth - graphWidth) / 2,
    y: minY - (nextHeight - graphHeight) / 2,
  };
}

function getConnectedNodeIds(edges: EdgePosition[], nodeId: string): Set<string> {
  const connected = new Set<string>();
  for (const edge of edges) {
    if (edge.source === nodeId) connected.add(edge.target);
    if (edge.target === nodeId) connected.add(edge.source);
  }
  return connected;
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}


function hashString(value: string): number {
  let hash = 0;
  for (let index = 0; index < value.length; index++) {
    const char = value.charCodeAt(index);
    hash = ((hash << 5) - hash) + char;
    hash = hash & hash;
  }
  return Math.abs(hash);
}

function seededRandom(seed: number): () => number {
  let state = seed;
  return () => {
    state = (state * 1664525 + 1013904223) & 0xffffffff;
    return (state >>> 0) / 0xffffffff;
  };
}

function applyLayout(
  rawNodes: Array<{ id: string; name: string; nodeType: string }>,
  layout: GraphLayout,
): NodePosition[] {
  switch (layout) {
    case "star":
      return positionNodesStar(rawNodes);
    case "neuron":
      return positionNodesNeuron(rawNodes);
    case "random":
      return positionNodesRandom(rawNodes);
    default:
      return positionNodesCircle(rawNodes);
  }
}

function positionNodesCircle(nodes: Array<{ id: string; name: string; nodeType: string }>): NodePosition[] {
  const width = 960;
  const height = 560;
  const centerX = width / 2;
  const centerY = height / 2;
  const radius = Math.min(width, height) * 0.36;

  return nodes.map((node, index) => {
    const angle = (Math.PI * 2 * index) / Math.max(nodes.length, 1);
    return {
      id: node.id,
      label: node.name.length > 28 ? node.name.slice(0, 25) + "..." : node.name,
      nodeType: node.nodeType,
      x: centerX + Math.cos(angle) * radius,
      y: centerY + Math.sin(angle) * radius,
    };
  });
}

function positionNodesStar(nodes: Array<{ id: string; name: string; nodeType: string }>): NodePosition[] {
  const width = 960;
  const height = 560;
  const centerX = width / 2;
  const centerY = height / 2;
  const arms = 5;
  const armLength = Math.min(width, height) * 0.35;
  const jitter = 18;

  return nodes.map((node, index) => {
    const armIndex = index % arms;
    const posInArm = Math.floor(index / arms);
    const armAngle = (Math.PI * 2 * armIndex) / arms - Math.PI / 2;
    const spread = 0.35;
    const distance = armLength * (0.25 + 0.75 * (posInArm / Math.max(Math.ceil(nodes.length / arms), 1)));
    const angle = armAngle + (posInArm % 2 === 0 ? spread : -spread) * Math.min(posInArm, 4);
    const rng = seededRandom(hashString(node.id));
    const jitterX = (rng() - 0.5) * jitter * 2;
    const jitterY = (rng() - 0.5) * jitter * 2;
    return {
      id: node.id,
      label: node.name.length > 28 ? node.name.slice(0, 25) + "..." : node.name,
      nodeType: node.nodeType,
      x: centerX + Math.cos(angle) * distance + jitterX,
      y: centerY + Math.sin(angle) * distance + jitterY,
    };
  });
}

function positionNodesNeuron(nodes: Array<{ id: string; name: string; nodeType: string }>): NodePosition[] {
  const width = 960;
  const height = 560;
  const centerX = width / 2;
  const centerY = height / 2;
  const maxRadius = Math.min(width, height) * 0.38;

  const sorted = [...nodes].sort((a, b) => {
    if (a.nodeType === "Repository") return -1;
    if (b.nodeType === "Repository") return 1;
    return a.name.localeCompare(b.name);
  });

  const total = sorted.length;
  return sorted.map((node, index) => {
    let x: number, y: number;
    if (node.nodeType === "Repository") {
      x = centerX;
      y = centerY;
    } else if (index < total * 0.25) {
      const angle = (Math.PI * 2 * index) / Math.max(total * 0.25, 1);
      const r = maxRadius * 0.3;
      x = centerX + Math.cos(angle) * r;
      y = centerY + Math.sin(angle) * r;
    } else if (index < total * 0.65) {
      const angle = (Math.PI * 2 * (index - total * 0.25)) / Math.max(total * 0.4, 1);
      const r = maxRadius * 0.6;
      x = centerX + Math.cos(angle) * r;
      y = centerY + Math.sin(angle) * r;
    } else {
      const angle = (Math.PI * 2 * (index - total * 0.65)) / Math.max(total * 0.35, 1);
      const r = maxRadius * 0.85;
      x = centerX + Math.cos(angle) * r;
      y = centerY + Math.sin(angle) * r;
    }
    return {
      id: node.id,
      label: node.name.length > 28 ? node.name.slice(0, 25) + "..." : node.name,
      nodeType: node.nodeType,
      x,
      y,
    };
  });
}

function positionNodesRandom(nodes: Array<{ id: string; name: string; nodeType: string }>): NodePosition[] {
  const width = 960;
  const height = 560;
  const margin = 70;

  return nodes.map((node) => {
    const rng = seededRandom(hashString(node.id + node.name));
    return {
      id: node.id,
      label: node.name.length > 28 ? node.name.slice(0, 25) + "..." : node.name,
      nodeType: node.nodeType,
      x: margin + rng() * (width - 2 * margin),
      y: margin + rng() * (height - 2 * margin),
    };
  });
}

function getNodeTypeCounts(nodes: Array<{ nodeType: string }>): Array<{ count: number; label: string }> {
  const counts = new Map<string, number>();
  for (const node of nodes) {
    counts.set(node.nodeType, (counts.get(node.nodeType) ?? 0) + 1);
  }
  return Array.from(counts.entries())
    .map(([label, count]) => ({ count, label }))
    .sort((a, b) => b.count - a.count || a.label.localeCompare(b.label));
}

function nodeClass(nodeType: string): string {
  if (nodeType.toLowerCase().includes("file")) return "fill-secondary stroke-primary";
  if (nodeType.toLowerCase().includes("folder")) return "fill-muted stroke-primary";
  return "fill-primary stroke-primary";
}
