<script lang="ts">
  import { onMount } from 'svelte';
  import {
    Database,
    Terminal,
    Activity,
    Layers,
    Server,
    ShieldAlert,
    Trash2,
    RefreshCw,
    Plus
  } from 'lucide-svelte';
  import { redisStore } from '../../state/redis.svelte';
  import { connectionStore } from '../../state/connection.svelte';
  import RedisKeyTree from './RedisKeyTree.svelte';
  import RedisKeyDetailView from './RedisKeyDetailView.svelte';
  import RedisCliConsole from './RedisCliConsole.svelte';
  import RedisServerInfoView from './RedisServerInfoView.svelte';
  import RedisNewKeyModal from './RedisNewKeyModal.svelte';

  let {
    tabId,
    connectionId
  }: {
    tabId: string;
    connectionId: string;
  } = $props();

  const redisState = $derived(redisStore.getState(connectionId));
  const connection = $derived(connectionStore.connections.find(c => c.id === connectionId));

  let isNewKeyModalOpen = $state(false);
  let treeWidth = $state(320);
  let isResizing = $state(false);

  onMount(() => {
    redisStore.loadKeys(connectionId);
  });

  function handleStartResize(e: MouseEvent) {
    isResizing = true;
    const startX = e.clientX;
    const startWidth = treeWidth;

    function onMouseMove(moveEvent: MouseEvent) {
      const delta = moveEvent.clientX - startX;
      treeWidth = Math.max(240, Math.min(600, startWidth + delta));
    }

    function onMouseUp() {
      isResizing = false;
      window.removeEventListener('mousemove', onMouseMove);
      window.removeEventListener('mouseup', onMouseUp);
    }

    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('mouseup', onMouseUp);
  }

  async function handleDeleteKey(key: string) {
    if (!confirm(`Are you sure you want to delete Redis key:\n"${key}"?`)) {
      return;
    }
    try {
      await redisStore.deleteKey(connectionId, key);
    } catch (err: any) {
      alert(`Failed to delete key: ${err?.message || err}`);
    }
  }

  async function handleFlushDb() {
    const db = redisState.selectedDb;
    const confirmed = confirm(
      `⚠️ WARNING: DANGER ZONE\n\nAre you sure you want to flush all ${redisState.totalKeys} keys in DB ${db}?\nThis action cannot be undone.`
    );
    if (!confirmed) return;

    try {
      await redisStore.flushCurrentDb(connectionId);
      alert(`Database ${db} flushed successfully.`);
    } catch (err: any) {
      alert(`Failed to flush database: ${err?.message || err}`);
    }
  }
</script>

<div class="h-full flex flex-col bg-zinc-950 text-zinc-100 select-none overflow-hidden">
  <!-- Top Navigation Header -->
  <div class="h-10 px-4 bg-zinc-900 border-b border-zinc-800 flex items-center justify-between shrink-0">
    <!-- Sub-view navigation tabs -->
    <div class="flex items-center gap-1">
      <button
        type="button"
        onclick={() => (redisState.activeSubView = 'browser')}
        class="flex items-center gap-1.5 px-3 py-1 text-xs rounded-md font-medium transition-colors cursor-pointer {redisState.activeSubView === 'browser' ? 'bg-zinc-800 text-indigo-300 shadow-sm' : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-850'}"
      >
        <Database class="w-3.5 h-3.5 text-indigo-400" />
        <span>Key Browser</span>
      </button>

      <button
        type="button"
        onclick={() => (redisState.activeSubView = 'cli')}
        class="flex items-center gap-1.5 px-3 py-1 text-xs rounded-md font-medium transition-colors cursor-pointer {redisState.activeSubView === 'cli' ? 'bg-zinc-800 text-emerald-300 shadow-sm' : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-850'}"
      >
        <Terminal class="w-3.5 h-3.5 text-emerald-400" />
        <span>CLI Console</span>
      </button>

      <button
        type="button"
        onclick={() => (redisState.activeSubView = 'info')}
        class="flex items-center gap-1.5 px-3 py-1 text-xs rounded-md font-medium transition-colors cursor-pointer {redisState.activeSubView === 'info' ? 'bg-zinc-800 text-cyan-300 shadow-sm' : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-850'}"
      >
        <Activity class="w-3.5 h-3.5 text-cyan-400" />
        <span>Server Telemetry</span>
      </button>
    </div>

    <!-- Connection Info Badge -->
    <div class="flex items-center gap-2 text-xs">
      <span class="flex items-center gap-1.5 px-2.5 py-0.5 rounded-full bg-zinc-950 border border-zinc-800 text-zinc-400 font-mono text-[11px]">
        <span class="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>
        <span>{connection?.name || 'Redis Node'}</span>
        <span class="text-zinc-600">({connection?.host || '127.0.0.1'}:{connection?.port || 6379})</span>
      </span>
    </div>
  </div>

  <!-- Main View Container -->
  <div class="flex-1 flex overflow-hidden">
    {#if redisState.activeSubView === 'browser'}
      <!-- Left Pane: Key Tree Explorer -->
      <div style="width: {treeWidth}px;" class="h-full shrink-0 flex flex-col">
        <RedisKeyTree
          {connectionId}
          onOpenNewKeyModal={() => (isNewKeyModalOpen = true)}
          onFlushDb={handleFlushDb}
        />
      </div>

      <!-- Resizer Handle -->
      <button
        type="button"
        aria-label="Resize key tree explorer"
        onmousedown={handleStartResize}
        class="w-1 hover:w-1.5 hover:bg-indigo-500 bg-zinc-800/80 cursor-col-resize transition-all shrink-0 select-none p-0 border-0 {isResizing ? 'bg-indigo-500' : ''}"
      ></button>

      <!-- Right Pane: Key Detail View -->
      <div class="flex-1 h-full overflow-hidden">
        <RedisKeyDetailView
          {connectionId}
          onDeleteKey={handleDeleteKey}
        />
      </div>
    {:else if redisState.activeSubView === 'cli'}
      <div class="flex-1 h-full overflow-hidden">
        <RedisCliConsole {connectionId} />
      </div>
    {:else if redisState.activeSubView === 'info'}
      <div class="flex-1 h-full overflow-hidden">
        <RedisServerInfoView {connectionId} />
      </div>
    {/if}
  </div>

  <!-- New Key Modal -->
  <RedisNewKeyModal
    {connectionId}
    isOpen={isNewKeyModalOpen}
    onClose={() => (isNewKeyModalOpen = false)}
  />
</div>
