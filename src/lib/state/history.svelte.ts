export interface QueryHistoryItem {
  id: string;
  sql: string;
  connectionId: string;
  connectionName: string;
  database: string;
  executedAt: number; // timestamp ms
  durationMs: number;
  status: 'success' | 'error';
  errorMessage?: string;
  rowCount?: number;
  isFavorite?: boolean;
  favoriteLabel?: string;
}

const STORAGE_KEY = 'fugdb_query_history';
const MAX_HISTORY_ITEMS = 500;

export class HistoryStore {
  items = $state<QueryHistoryItem[]>([]);
  isOpen = $state<boolean>(false);
  searchQuery = $state<string>('');
  filterStatus = $state<'all' | 'favorite' | 'success' | 'error'>('all');
  selectedConnectionFilter = $state<string>('all'); // 'all' or connectionId

  constructor() {
    this.loadFromStorage();
  }

  private loadFromStorage() {
    try {
      const data = localStorage.getItem(STORAGE_KEY);
      if (data) {
        this.items = JSON.parse(data);
      }
    } catch (err) {
      console.error('Failed to load query history from storage:', err);
      this.items = [];
    }
  }

  private saveToStorage() {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(this.items));
    } catch (err) {
      console.error('Failed to save query history to storage:', err);
    }
  }

  addEntry(entry: Omit<QueryHistoryItem, 'id' | 'isFavorite'> & { isFavorite?: boolean }) {
    // Avoid exact duplicate immediately following previous entry
    const first = this.items[0];
    if (
      first &&
      first.sql.trim() === entry.sql.trim() &&
      first.connectionId === entry.connectionId &&
      Date.now() - first.executedAt < 2000
    ) {
      return;
    }

    const newItem: QueryHistoryItem = {
      id: `hist-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`,
      isFavorite: false,
      ...entry,
    };

    // Keep up to MAX_HISTORY_ITEMS, preserving favorite items if pruned
    this.items = [newItem, ...this.items.slice(0, MAX_HISTORY_ITEMS - 1)];
    this.saveToStorage();
  }

  toggleFavorite(id: string) {
    const item = this.items.find(i => i.id === id);
    if (item) {
      item.isFavorite = !item.isFavorite;
      this.saveToStorage();
    }
  }

  setFavoriteLabel(id: string, label: string) {
    const item = this.items.find(i => i.id === id);
    if (item) {
      item.favoriteLabel = label;
      this.saveToStorage();
    }
  }

  removeEntry(id: string) {
    this.items = this.items.filter(i => i.id !== id);
    this.saveToStorage();
  }

  clearHistory() {
    // Preserve favorites when clearing regular history
    this.items = this.items.filter(i => i.isFavorite);
    this.saveToStorage();
  }

  clearAll() {
    this.items = [];
    this.saveToStorage();
  }

  toggleDrawer() {
    this.isOpen = !this.isOpen;
  }

  filteredItems = $derived.by(() => {
    let result = this.items;

    // Filter by status / favorite
    if (this.filterStatus === 'favorite') {
      result = result.filter(i => i.isFavorite);
    } else if (this.filterStatus === 'success') {
      result = result.filter(i => i.status === 'success');
    } else if (this.filterStatus === 'error') {
      result = result.filter(i => i.status === 'error');
    }

    // Filter by connection
    if (this.selectedConnectionFilter !== 'all') {
      result = result.filter(i => i.connectionId === this.selectedConnectionFilter);
    }

    // Filter by search query
    if (this.searchQuery.trim()) {
      const q = this.searchQuery.toLowerCase();
      result = result.filter(
        i =>
          i.sql.toLowerCase().includes(q) ||
          i.database.toLowerCase().includes(q) ||
          i.connectionName.toLowerCase().includes(q) ||
          (i.favoriteLabel && i.favoriteLabel.toLowerCase().includes(q))
      );
    }

    return result;
  });

  favoriteCount = $derived(this.items.filter(i => i.isFavorite).length);
  successCount = $derived(this.items.filter(i => i.status === 'success').length);
  errorCount = $derived(this.items.filter(i => i.status === 'error').length);
}

export const historyStore = new HistoryStore();
