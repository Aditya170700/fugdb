export type InspectorTab = 'auto' | 'json' | 'datetime' | 'binary' | 'text' | 'image';

export interface InspectorTarget {
  columnName: string;
  dataType?: string;
  tableName?: string;
  rowKey?: string;
  colIdx?: number;
  isInserted?: boolean;
  tempId?: string;
  value: any;
  onApply?: (newValue: any) => void;
}

export class InspectorStore {
  isOpen = $state<boolean>(false);
  activeTab = $state<InspectorTab>('auto');
  target = $state<InspectorTarget | null>(null);

  open(target: InspectorTarget, preferredTab: InspectorTab = 'auto') {
    this.target = target;
    this.activeTab = preferredTab === 'auto' ? this.detectType(target.value, target.dataType) : preferredTab;
    this.isOpen = true;
  }

  close() {
    this.isOpen = false;
    this.target = null;
  }

  detectType(val: any, dataType?: string): InspectorTab {
    if (val === null || val === undefined) return 'text';

    const dt = (dataType || '').toLowerCase();
    if (dt.includes('json')) return 'json';
    if (dt.includes('time') || dt.includes('date')) return 'datetime';
    if (dt.includes('uuid') || dt.includes('binary') || dt.includes('bytea') || dt.includes('blob')) return 'binary';

    // Check if object
    if (typeof val === 'object') return 'json';

    const str = String(val).trim();

    // Check if JSON string
    if ((str.startsWith('{') && str.endsWith('}')) || (str.startsWith('[') && str.endsWith(']'))) {
      try {
        JSON.parse(str);
        return 'json';
      } catch {}
    }

    // Check if SVG or Image Data URL
    if (str.startsWith('<svg') && str.endsWith('</svg>')) return 'image';
    if (str.startsWith('data:image/') || str.match(/\.(png|jpe?g|gif|webp|svg)(\?.*)?$/i)) return 'image';

    // Check if Date string
    if (
      str.match(/^\d{4}-\d{2}-\d{2}(T|\s)\d{2}:\d{2}:\d{2}/) ||
      str.match(/^\d{4}-\d{2}-\d{2}$/) ||
      (str.length >= 10 && str.length <= 13 && !isNaN(Number(str)) && Number(str) > 946684800) // Timestamp > year 2000
    ) {
      const parsed = Date.parse(str);
      if (!isNaN(parsed)) return 'datetime';
    }

    // Check if UUID
    if (str.match(/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i)) {
      return 'binary';
    }

    // Check if Base64 or Hex
    if (str.length > 8 && str.length % 2 === 0 && str.match(/^[0-9a-fA-F]+$/)) {
      return 'binary';
    }

    return 'text';
  }
}

export const inspectorStore = new InspectorStore();
