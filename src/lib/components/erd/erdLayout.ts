import type { TableItem, RelationEdge } from '$lib/api/types';
import type { Node, Edge } from '@xyflow/svelte';
import { Position, MarkerType } from '@xyflow/svelte';

export interface ErdNodeData extends Record<string, unknown> {
  name: string;
  schema: string;
  tableType: string;
  columns: any[];
  rowCountEstimate?: number;
  onOpenTable?: (tableName: string, schema?: string) => void;
  onQueryTable?: (tableName: string, schema?: string) => void;
}

export function generateErdElements(
  tables: TableItem[],
  relations: RelationEdge[] = [],
  callbacks?: {
    onOpenTable?: (tableName: string, schema?: string) => void;
    onQueryTable?: (tableName: string, schema?: string) => void;
  }
): { nodes: Node<ErdNodeData>[]; edges: Edge[] } {
  // 1. Build adjacency map to calculate topological layers
  const tableNames = new Set(tables.map(t => t.name));
  const incomingDegree = new Map<string, number>();
  const outgoingNeighbors = new Map<string, string[]>();

  tables.forEach(t => {
    incomingDegree.set(t.name, 0);
    outgoingNeighbors.set(t.name, []);
  });

  relations.forEach(rel => {
    if (tableNames.has(rel.fromTable) && tableNames.has(rel.toTable)) {
      outgoingNeighbors.get(rel.fromTable)?.push(rel.toTable);
      incomingDegree.set(rel.toTable, (incomingDegree.get(rel.toTable) || 0) + 1);
    }
  });

  // 2. Assign layers based on depth
  const tableLayers = new Map<string, number>();
  const visited = new Set<string>();

  // Tables with 0 incoming FKs are Layer 0 (roots)
  const queue: Array<{ name: string; layer: number }> = [];
  tables.forEach(t => {
    if ((incomingDegree.get(t.name) || 0) === 0) {
      queue.push({ name: t.name, layer: 0 });
      tableLayers.set(t.name, 0);
      visited.add(t.name);
    }
  });

  // BFS to assign layers
  while (queue.length > 0) {
    const { name, layer } = queue.shift()!;
    const neighbors = outgoingNeighbors.get(name) || [];
    for (const neighbor of neighbors) {
      const nextLayer = layer + 1;
      const currentAssigned = tableLayers.get(neighbor) || 0;
      if (nextLayer > currentAssigned) {
        tableLayers.set(neighbor, nextLayer);
      }
      if (!visited.has(neighbor)) {
        visited.add(neighbor);
        queue.push({ name: neighbor, layer: nextLayer });
      }
    }
  }

  // Fallback for remaining unassigned tables (disconnected components)
  tables.forEach(t => {
    if (!tableLayers.has(t.name)) {
      tableLayers.set(t.name, 0);
    }
  });

  // 3. Group tables by layer for coordinate calculation
  const layerGroups = new Map<number, TableItem[]>();
  tables.forEach(t => {
    const layer = tableLayers.get(t.name) || 0;
    if (!layerGroups.has(layer)) {
      layerGroups.set(layer, []);
    }
    layerGroups.get(layer)!.push(t);
  });

  const NODE_WIDTH = 280;
  const X_SPACING = 380;
  const Y_SPACING = 20;

  const nodes: Node<ErdNodeData>[] = [];

  // Track vertical offsets per layer
  const sortedLayers = Array.from(layerGroups.keys()).sort((a, b) => a - b);
  sortedLayers.forEach(layerIdx => {
    const layerTables = layerGroups.get(layerIdx) || [];
    let currentY = 40;

    layerTables.forEach(t => {
      const columnCount = t.columns?.length || 3;
      const estimatedHeight = 50 + columnCount * 28 + 20;

      nodes.push({
        id: t.name,
        type: 'tableNode',
        position: {
          x: layerIdx * X_SPACING + 40,
          y: currentY
        },
        data: {
          name: t.name,
          schema: t.schema || 'public',
          tableType: t.tableType || 'table',
          columns: t.columns || [],
          rowCountEstimate: t.rowCountEstimate,
          onOpenTable: callbacks?.onOpenTable,
          onQueryTable: callbacks?.onQueryTable
        }
      });

      currentY += estimatedHeight + Y_SPACING;
    });
  });

  // 4. Generate Edges from Relations
  const edges: Edge[] = [];
  relations.forEach(rel => {
    if (tableNames.has(rel.fromTable) && tableNames.has(rel.toTable)) {
      edges.push({
        id: rel.id || `fk_${rel.fromTable}_${rel.fromColumn}_to_${rel.toTable}_${rel.toColumn}`,
        source: rel.fromTable,
        target: rel.toTable,
        sourceHandle: `${rel.fromTable}_${rel.fromColumn}_right`,
        targetHandle: `${rel.toTable}_${rel.toColumn}_left`,
        label: `${rel.fromColumn} → ${rel.toColumn}`,
        type: 'smoothstep',
        animated: true,
        style: 'stroke: #6366f1; stroke-width: 2px;',
        markerEnd: {
          type: MarkerType.ArrowClosed,
          width: 14,
          height: 14,
          color: '#6366f1'
        }
      });
    }
  });

  return { nodes, edges };
}
