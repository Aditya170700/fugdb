<script lang="ts">
  import {
    Search,
    RefreshCw,
    Plus,
    Trash2,
    Folder,
    FolderOpen,
    Key,
    Clock,
    Database,
    ChevronRight,
    ChevronDown,
    ListTree,
    List,
    AlertTriangle,
    Layers,
    Server,
    Terminal
  } from 'lucide-svelte';
  import { redisStore } from '../../state/redis.svelte';
  import type { RedisKeyItem, RedisKeyType } from '../../api/types';
  import CustomSelect from '../ui/CustomSelect.svelte';

  let {
    connectionId,
    onOpenNewKeyModal,
    onFlushDb
  }: {
    connectionId: string;
    onOpenNewKeyModal: () => void;
    onFlushDb: () => void;
  } = $props();

  const redisState = $derived(redisStore.getState(connectionId));

  let viewMode = $state<'tree' | 'flat'>('tree');
  let searchInput = $state('');
  let expandedFolders = $state<Record<string, boolean>>({});

  // DB options 0 - 15
  const dbOptions = Array.from({ length: 16 }, (_, i) => ({
    value: i.toString(),
    label: `DB ${i}${i === 0 ? ' (Default)' : ''}`
  }));

  function handleDbChange(dbStr: string) {
    const dbIndex = parseInt(dbStr, 10);
    redisStore.loadKeys(connectionId, redisState.searchPattern, dbIndex);
  }

  function handleSearchSubmit() {
    const pattern = searchInput.trim() ? (searchInput.includes('*') ? searchInput : `*${searchInput}*`) : '*';
    redisStore.loadKeys(connectionId, pattern);
  }

  function handleKeySelect(key: string) {
    redisStore.selectKey(connectionId, key);
  }

  function toggleFolder(path: string) {
    expandedFolders[path] = !expandedFolders[path];
  }

  function getTypeBadge(type: string) {
    switch (type.toLowerCase()) {
      case 'string':
        return { label: 'STR', color: 'bg-blue-500/15 text-blue-400 border-blue-500/30' };
      case 'hash':
        return { label: 'HASH', color: 'bg-purple-500/15 text-purple-400 border-purple-500/30' };
      case 'list':
        return { label: 'LIST', color: 'bg-emerald-500/15 text-emerald-400 border-emerald-500/30' };
      case 'set':
        return { label: 'SET', color: 'bg-amber-500/15 text-amber-400 border-amber-500/30' };
      case 'zset':
        return { label: 'ZSET', color: 'bg-rose-500/15 text-rose-400 border-rose-500/30' };
      case 'stream':
        return { label: 'STRM', color: 'bg-cyan-500/15 text-cyan-400 border-cyan-500/30' };
      default:
        return { label: type.slice(0, 4).toUpperCase(), color: 'bg-zinc-500/15 text-zinc-400 border-zinc-500/30' };
    }
  }

  function formatTtl(ttl: number): string {
    if (ttl === -1) return '∞';
    if (ttl === -2) return 'exp';
    if (ttl < 60) return `${ttl}s`;
    if (ttl < 3600) return `${Math.floor(ttl / 60)}m`;
    if (ttl < 86400) return `${Math.floor(ttl / 3600)}h`;
    return `${Math.floor(ttl / 86400)}d`;
  }

  // Build hierarchical namespace tree
  interface TreeNode {
    name: string;
    path: string;
    isKey: boolean;
    keyItem?: RedisKeyItem;
    children: Record<string, TreeNode>;
  }

  const keyTree = $derived.by(() => {
    const root: Record<string, TreeNode> = {};

    for (const item of redisState.keys) {
      const parts = item.key.split(':');
      let currentLevel = root;
      let pathAcc = '';

      parts.forEach((part, index) => {
        pathAcc = pathAcc ? `${pathAcc}:${part}` : part;
        const isLast = index === parts.length - 1;

        if (!currentLevel[part]) {
          currentLevel[part] = {
            name: part,
            path: pathAcc,
            isKey: isLast,
            keyItem: isLast ? item : undefined,
            children: {}
          };
        } else if (isLast) {
          currentLevel[part].isKey = true;
          currentLevel[part].keyItem = item;
        }

        currentLevel = currentLevel[part].children;
      });
    }

    return root;
  });
</script>

{#snippet treeNodeRenderer(node: TreeNode, depth: number)}
  {@const hasChildren = Object.keys(node.children).length > 0}
  {@const isExpanded = expandedFolders[node.path] ?? (depth < 1)}
  {@const isSelected = redisState.selectedKeyName === node.path}

  {#if hasChildren}
    <div>
      <button
        type="button"
        onclick={() => toggleFolder(node.path)}
        class="w-full flex items-center gap-1.5 px-2 py-1 text-xs text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/60 rounded cursor-pointer transition-colors text-left"
        style="padding-left: {depth * 12 + 8}px;"
      >
        {#if isExpanded}
          <ChevronDown class="w-3.5 h-3.5 text-zinc-500 shrink-0" />
          <FolderOpen class="w-3.5 h-3.5 text-amber-400/80 shrink-0" />
        {:else}
          <ChevronRight class="w-3.5 h-3.5 text-zinc-500 shrink-0" />
          <Folder class="w-3.5 h-3.5 text-amber-400/80 shrink-0" />
        {/if}
        <span class="font-medium text-zinc-300 truncate">{node.name}</span>
        <span class="text-[10px] text-zinc-600 ml-auto font-mono">({Object.keys(node.children).length})</span>
      </button>

      {#if isExpanded}
        <div class="flex flex-col">
          {#each Object.values(node.children) as child (child.path)}
            {@render treeNodeRenderer(child, depth + 1)}
          {/each}
        </div>
      {/if}
    </div>
  {:else if node.isKey && node.keyItem}
    {@const badge = getTypeBadge(node.keyItem.keyType)}
    <button
      type="button"
      onclick={() => handleKeySelect(node.path)}
      class="w-full flex items-center gap-2 px-2 py-1 text-xs rounded transition-colors text-left cursor-pointer {isSelected ? 'bg-indigo-600/20 text-indigo-300 border-l-2 border-indigo-500' : 'text-zinc-300 hover:bg-zinc-800/50 hover:text-zinc-100'}"
      style="padding-left: {depth * 12 + 8}px;"
    >
      <span class="text-[9px] px-1 py-0.2 rounded font-mono font-bold border uppercase shrink-0 {badge.color}">
        {badge.label}
      </span>
      <span class="font-mono text-[11px] truncate flex-1" title={node.path}>{node.name}</span>
      {#if node.keyItem.ttl !== -1}
        <span class="text-[10px] text-amber-400/80 font-mono shrink-0 flex items-center gap-0.5">
          <Clock class="w-2.5 h-2.5" />
          {formatTtl(node.keyItem.ttl)}
        </span>
      {/if}
    </button>
  {/if}
{/snippet}

<div class="h-full flex flex-col bg-zinc-900 border-r border-zinc-800 select-none">
  <!-- DB Selector & Stats Header -->
  <div class="p-2.5 border-b border-zinc-800/80 flex flex-col gap-2 bg-zinc-900/90">
    <div class="flex items-center justify-between gap-1.5">
      <div class="flex-1">
        <CustomSelect
          value={redisState.selectedDb.toString()}
          options={dbOptions}
          onchange={handleDbChange}
          placeholder="Select DB"
        />
      </div>
      <button
        type="button"
        onclick={() => redisStore.loadKeys(connectionId)}
        class="p-1.5 text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800 rounded transition-colors cursor-pointer"
        title="Refresh Keys"
      >
        <RefreshCw class="w-3.5 h-3.5 {redisState.isLoadingKeys ? 'animate-spin text-indigo-400' : ''}" />
      </button>
      <button
        type="button"
        onclick={onOpenNewKeyModal}
        class="flex items-center gap-1 px-2 py-1 text-xs bg-indigo-600 hover:bg-indigo-500 text-white rounded font-medium transition-colors shadow-sm cursor-pointer"
        title="Create New Key"
      >
        <Plus class="w-3.5 h-3.5" />
        <span>New</span>
      </button>
    </div>

    <!-- Search Input Filter -->
    <div class="relative flex items-center">
      <Search class="w-3.5 h-3.5 absolute left-2 text-zinc-500 pointer-events-none" />
      <input
        type="text"
        bind:value={searchInput}
        onkeydown={(e) => e.key === 'Enter' && handleSearchSubmit()}
        placeholder="Search pattern (e.g. user:*)"
        class="w-full bg-zinc-950 border border-zinc-800 rounded pl-7 pr-7 py-1 text-xs text-zinc-200 placeholder-zinc-500 focus:outline-none focus:border-indigo-500/70 font-mono"
      />
      {#if searchInput}
        <button
          type="button"
          onclick={() => { searchInput = ''; redisStore.loadKeys(connectionId, '*'); }}
          class="absolute right-2 text-zinc-500 hover:text-zinc-300 text-xs cursor-pointer"
        >
          ×
        </button>
      {/if}
    </div>

    <!-- View Mode Toggle & Counter -->
    <div class="flex items-center justify-between text-[11px] text-zinc-400 pt-0.5">
      <div class="flex items-center gap-1 font-mono">
        <span class="text-zinc-200 font-semibold">{redisState.keys.length}</span>
        <span class="text-zinc-500">/ {redisState.totalKeys} keys</span>
      </div>

      <div class="flex items-center gap-1 bg-zinc-950 p-0.5 rounded border border-zinc-800">
        <button
          type="button"
          onclick={() => (viewMode = 'tree')}
          class="p-1 rounded transition-colors cursor-pointer {viewMode === 'tree' ? 'bg-zinc-800 text-indigo-300' : 'text-zinc-500 hover:text-zinc-300'}"
          title="Tree Namespace View"
        >
          <ListTree class="w-3 h-3" />
        </button>
        <button
          type="button"
          onclick={() => (viewMode = 'flat')}
          class="p-1 rounded transition-colors cursor-pointer {viewMode === 'flat' ? 'bg-zinc-800 text-indigo-300' : 'text-zinc-500 hover:text-zinc-300'}"
          title="Flat List View"
        >
          <List class="w-3 h-3" />
        </button>
      </div>
    </div>
  </div>

  <!-- Key List / Tree Area -->
  <div class="flex-1 overflow-y-auto p-1.5 space-y-0.5 custom-scrollbar">
    {#if redisState.isLoadingKeys}
      <div class="flex flex-col items-center justify-center py-12 gap-2 text-zinc-500">
        <RefreshCw class="w-5 h-5 animate-spin text-indigo-400" />
        <span class="text-xs">Scanning Redis keys...</span>
      </div>
    {:else if redisState.keys.length === 0}
      <div class="flex flex-col items-center justify-center py-12 px-4 text-center text-zinc-500 gap-2">
        <Database class="w-8 h-8 text-zinc-700 stroke-1" />
        <p class="text-xs text-zinc-400 font-medium">No keys found in DB {redisState.selectedDb}</p>
        <p class="text-[11px] text-zinc-600">Add a new key or change the search filter pattern.</p>
        <button
          type="button"
          onclick={onOpenNewKeyModal}
          class="mt-2 text-xs px-2.5 py-1 bg-zinc-800 hover:bg-zinc-700 text-zinc-200 rounded border border-zinc-700 transition-colors cursor-pointer"
        >
          + Add First Key
        </button>
      </div>
    {:else if viewMode === 'tree'}
      {#each Object.values(keyTree) as node (node.path)}
        {@render treeNodeRenderer(node, 0)}
      {/each}
    {:else}
      <!-- Flat List View -->
      {#each redisState.keys as item (item.key)}
        {@const badge = getTypeBadge(item.keyType)}
        {@const isSelected = redisState.selectedKeyName === item.key}
        <button
          type="button"
          onclick={() => handleKeySelect(item.key)}
          class="w-full flex items-center gap-2 px-2 py-1 text-xs rounded transition-colors text-left cursor-pointer {isSelected ? 'bg-indigo-600/20 text-indigo-300 border-l-2 border-indigo-500' : 'text-zinc-300 hover:bg-zinc-800/50 hover:text-zinc-100'}"
        >
          <span class="text-[9px] px-1 py-0.2 rounded font-mono font-bold border uppercase shrink-0 {badge.color}">
            {badge.label}
          </span>
          <span class="font-mono text-[11px] truncate flex-1" title={item.key}>{item.key}</span>
          {#if item.ttl !== -1}
            <span class="text-[10px] text-amber-400/80 font-mono shrink-0 flex items-center gap-0.5">
              <Clock class="w-2.5 h-2.5" />
              {formatTtl(item.ttl)}
            </span>
          {/if}
        </button>
      {/each}
    {/if}
  </div>

  <!-- Bottom Toolbar: Flush DB -->
  <div class="p-2 border-t border-zinc-800/80 bg-zinc-950/60 flex items-center justify-between">
    <span class="text-[10px] text-zinc-600 font-mono">FugDB Key-Value Inspector</span>
    <button
      type="button"
      onclick={onFlushDb}
      class="flex items-center gap-1 px-1.5 py-0.5 text-[10px] text-rose-400 hover:bg-rose-950/40 hover:text-rose-300 rounded border border-rose-900/30 transition-colors cursor-pointer"
      title="Flush all keys in DB {redisState.selectedDb}"
    >
      <Trash2 class="w-2.5 h-2.5" />
      <span>Flush DB</span>
    </button>
  </div>
</div>
