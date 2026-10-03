<script lang="ts">
  import {
    RefreshCw,
    Trash2,
    Clock,
    Edit2,
    Check,
    X,
    Save,
    Plus,
    Copy,
    Cpu,
    ArrowUpRight,
    ArrowDownLeft,
    Search,
    Code,
    FileJson,
    Binary,
    AlertCircle,
    SlidersHorizontal
  } from 'lucide-svelte';
  import { redisStore } from '../../state/redis.svelte';
  import type { RedisKeyDetail, RedisZSetMember, RedisStreamEntry } from '../../api/types';

  let {
    connectionId,
    onDeleteKey
  }: {
    connectionId: string;
    onDeleteKey: (key: string) => void;
  } = $props();

  const redisState = $derived(redisStore.getState(connectionId));
  const detail = $derived(redisState.selectedKeyDetail);

  // Key rename state
  let isRenaming = $state(false);
  let renameValue = $state('');

  // TTL edit state
  let isEditingTtl = $state(false);
  let ttlInput = $state<number>(-1);

  // String viewer state
  let stringViewMode = $state<'raw' | 'json' | 'base64'>('raw');
  let stringContent = $state('');
  let isStringDirty = $state(false);
  let copyFeedback = $state(false);

  // Hash state
  let hashFilter = $state('');
  let newHashField = $state('');
  let newHashValue = $state('');
  let isAddingHashField = $state(false);
  let editingHashField = $state<string | null>(null);
  let editingHashValue = $state('');

  // List state
  let newListItem = $state('');
  let listPushPos = $state<'left' | 'right'>('right');
  let isAddingListItem = $state(false);

  // Set state
  let setFilter = $state('');
  let newSetMember = $state('');
  let isAddingSetMember = $state(false);

  // ZSet state
  let zsetFilter = $state('');
  let newZSetMember = $state('');
  let newZSetScore = $state<number>(1.0);
  let isAddingZSetMember = $state(false);
  let editingZSetMember = $state<string | null>(null);
  let editingZSetScore = $state<number>(0);

  // Stream state
  let streamFilter = $state('');

  $effect(() => {
    if (detail) {
      renameValue = detail.key;
      ttlInput = detail.ttl;
      if (detail.valueString !== undefined && detail.valueString !== null) {
        stringContent = detail.valueString;
        isStringDirty = false;
        // Auto detect JSON
        try {
          JSON.parse(detail.valueString);
          stringViewMode = 'json';
        } catch {
          stringViewMode = 'raw';
        }
      }
    }
  });

  async function handleRenameSubmit() {
    if (!detail || !renameValue.trim() || renameValue === detail.key) {
      isRenaming = false;
      return;
    }
    try {
      await redisStore.renameKey(connectionId, detail.key, renameValue.trim());
      isRenaming = false;
    } catch (err) {
      alert(`Rename failed: ${err}`);
    }
  }

  async function handleTtlSubmit() {
    if (!detail) return;
    try {
      await redisStore.updateTtl(connectionId, detail.key, ttlInput);
      isEditingTtl = false;
    } catch (err) {
      alert(`TTL update failed: ${err}`);
    }
  }

  async function handleSaveString() {
    if (!detail) return;
    try {
      await redisStore.updateStringValue(connectionId, detail.key, stringContent, detail.ttl > 0 ? detail.ttl : undefined);
      isStringDirty = false;
    } catch (err) {
      alert(`Save failed: ${err}`);
    }
  }

  function handleFormatJson() {
    try {
      const parsed = JSON.parse(stringContent);
      stringContent = JSON.stringify(parsed, null, 2);
      isStringDirty = true;
    } catch (e) {
      alert('Invalid JSON format');
    }
  }

  function handleCopyValue(text: string) {
    navigator.clipboard.writeText(text);
    copyFeedback = true;
    setTimeout(() => (copyFeedback = false), 1500);
  }

  // Hash handlers
  async function handleAddHashField() {
    if (!detail || !newHashField.trim()) return;
    try {
      await redisStore.setHashField(connectionId, detail.key, newHashField.trim(), newHashValue);
      newHashField = '';
      newHashValue = '';
      isAddingHashField = false;
    } catch (err) {
      alert(`Failed to add hash field: ${err}`);
    }
  }

  async function handleSaveHashFieldEdit() {
    if (!detail || !editingHashField) return;
    try {
      await redisStore.setHashField(connectionId, detail.key, editingHashField, editingHashValue);
      editingHashField = null;
    } catch (err) {
      alert(`Failed to edit hash field: ${err}`);
    }
  }

  async function handleDeleteHashField(field: string) {
    if (!detail) return;
    if (!confirm(`Delete field "${field}"?`)) return;
    try {
      await redisStore.deleteHashField(connectionId, detail.key, field);
    } catch (err) {
      alert(`Failed to delete hash field: ${err}`);
    }
  }

  // List handlers
  async function handleAddListItem() {
    if (!detail || !newListItem.trim()) return;
    try {
      await redisStore.pushListElement(connectionId, detail.key, newListItem.trim(), listPushPos);
      newListItem = '';
      isAddingListItem = false;
    } catch (err) {
      alert(`Failed to push list item: ${err}`);
    }
  }

  async function handleRemoveListItem(value: string) {
    if (!detail) return;
    if (!confirm('Remove this element from list?')) return;
    try {
      await redisStore.removeListElement(connectionId, detail.key, value);
    } catch (err) {
      alert(`Failed to remove element: ${err}`);
    }
  }

  // Set handlers
  async function handleAddSetMember() {
    if (!detail || !newSetMember.trim()) return;
    try {
      await redisStore.addSetMember(connectionId, detail.key, newSetMember.trim());
      newSetMember = '';
      isAddingSetMember = false;
    } catch (err) {
      alert(`Failed to add set member: ${err}`);
    }
  }

  async function handleRemoveSetMember(member: string) {
    if (!detail) return;
    if (!confirm(`Remove member "${member}"?`)) return;
    try {
      await redisStore.removeSetMember(connectionId, detail.key, member);
    } catch (err) {
      alert(`Failed to remove member: ${err}`);
    }
  }

  // ZSet handlers
  async function handleAddZSetMember() {
    if (!detail || !newZSetMember.trim()) return;
    try {
      await redisStore.addZSetMember(connectionId, detail.key, newZSetMember.trim(), newZSetScore);
      newZSetMember = '';
      newZSetScore = 1.0;
      isAddingZSetMember = false;
    } catch (err) {
      alert(`Failed to add zset member: ${err}`);
    }
  }

  async function handleSaveZSetScoreEdit() {
    if (!detail || !editingZSetMember) return;
    try {
      await redisStore.addZSetMember(connectionId, detail.key, editingZSetMember, editingZSetScore);
      editingZSetMember = null;
    } catch (err) {
      alert(`Failed to update score: ${err}`);
    }
  }

  async function handleRemoveZSetMember(member: string) {
    if (!detail) return;
    if (!confirm(`Remove member "${member}"?`)) return;
    try {
      await redisStore.removeZSetMember(connectionId, detail.key, member);
    } catch (err) {
      alert(`Failed to remove zset member: ${err}`);
    }
  }

  function getTypeBadge(type: string) {
    switch (type.toLowerCase()) {
      case 'string':
        return { label: 'STRING', color: 'bg-blue-500/10 text-blue-700 dark:text-blue-400 border-blue-200 dark:border-blue-500/30' };
      case 'hash':
        return { label: 'HASH', color: 'bg-purple-500/10 text-purple-700 dark:text-purple-400 border-purple-200 dark:border-purple-500/30' };
      case 'list':
        return { label: 'LIST', color: 'bg-emerald-500/10 text-emerald-800 dark:text-emerald-400 border-emerald-200 dark:border-emerald-500/30' };
      case 'set':
        return { label: 'SET', color: 'bg-amber-500/10 text-amber-800 dark:text-amber-400 border-amber-200 dark:border-amber-500/30' };
      case 'zset':
        return { label: 'ZSET (SORTED SET)', color: 'bg-rose-500/10 text-rose-800 dark:text-rose-400 border-rose-200 dark:border-rose-500/30' };
      case 'stream':
        return { label: 'STREAM', color: 'bg-cyan-500/10 text-cyan-800 dark:text-cyan-400 border-cyan-200 dark:border-cyan-500/30' };
      default:
        return { label: type.toUpperCase(), color: 'bg-slate-500/10 text-slate-700 dark:text-zinc-400 border-slate-200 dark:border-zinc-500/30' };
    }
  }
</script>

{#if redisState.isLoadingDetail}
  <div class="h-full flex flex-col items-center justify-center gap-3 text-slate-500 dark:text-zinc-500 bg-surface-950 dark:bg-zinc-950">
    <RefreshCw class="w-6 h-6 animate-spin text-indigo-600 dark:text-indigo-400" />
    <span class="text-xs">Loading key details...</span>
  </div>
{:else if !detail}
  <div class="h-full flex flex-col items-center justify-center gap-3 text-slate-500 dark:text-zinc-500 bg-surface-950 dark:bg-zinc-950 p-6 text-center">
    <div class="p-3 bg-surface-900 dark:bg-zinc-900 rounded-full border border-slate-200 dark:border-zinc-800">
      <SlidersHorizontal class="w-6 h-6 text-slate-400 dark:text-zinc-600" />
    </div>
    <p class="text-sm font-medium text-slate-700 dark:text-zinc-400">No Key Selected</p>
    <p class="text-xs text-slate-500 dark:text-zinc-600 max-w-sm">Select a key from the left explorer tree or create a new key to inspect its values and TTL.</p>
  </div>
{:else}
  {@const badge = getTypeBadge(detail.keyType)}

  <div class="h-full flex flex-col bg-surface-950 dark:bg-zinc-950 text-slate-800 dark:text-zinc-200 overflow-hidden">
    <!-- Header Top Toolbar -->
    <div class="p-3.5 border-b border-slate-200 dark:border-zinc-800/80 bg-surface-900 dark:bg-zinc-900/60 flex flex-col gap-2.5 shrink-0">
      <div class="flex items-center justify-between gap-3">
        <!-- Key Name / Inline Rename -->
        <div class="flex items-center gap-2 flex-1 min-w-0">
          <span class="text-[10px] px-2 py-0.5 rounded font-mono font-bold border uppercase shrink-0 {badge.color}">
            {badge.label}
          </span>

          {#if isRenaming}
            <div class="flex items-center gap-1.5 flex-1 max-w-md">
              <input
                type="text"
                bind:value={renameValue}
                onkeydown={(e) => e.key === 'Enter' && handleRenameSubmit()}
                class="w-full bg-surface-950 dark:bg-zinc-950 border border-indigo-500 rounded px-2 py-1 text-xs font-mono text-slate-900 dark:text-zinc-100 focus:outline-none"
                autofocus
              />
              <button
                type="button"
                onclick={handleRenameSubmit}
                class="p-1 bg-indigo-600 hover:bg-indigo-500 text-white rounded text-xs cursor-pointer"
                title="Confirm Rename"
              >
                <Check class="w-3.5 h-3.5" />
              </button>
              <button
                type="button"
                onclick={() => (isRenaming = false)}
                class="p-1 bg-surface-800 dark:bg-zinc-800 hover:bg-surface-700 dark:hover:bg-zinc-700 text-slate-700 dark:text-zinc-300 rounded text-xs cursor-pointer"
                title="Cancel"
              >
                <X class="w-3.5 h-3.5" />
              </button>
            </div>
          {:else}
            <div class="flex items-center gap-2 truncate">
              <span class="font-mono text-sm font-semibold text-slate-900 dark:text-zinc-100 truncate select-all" title={detail.key}>
                {detail.key}
              </span>
              <button
                type="button"
                onclick={() => { isRenaming = true; renameValue = detail.key; }}
                class="p-1 text-slate-400 hover:text-slate-700 dark:text-zinc-500 dark:hover:text-zinc-300 hover:bg-surface-800 dark:hover:bg-zinc-800 rounded transition-colors cursor-pointer"
                title="Rename Key"
              >
                <Edit2 class="w-3 h-3" />
              </button>
              <button
                type="button"
                onclick={() => handleCopyValue(detail.key)}
                class="p-1 text-slate-400 hover:text-slate-700 dark:text-zinc-500 dark:hover:text-zinc-300 hover:bg-surface-800 dark:hover:bg-zinc-800 rounded transition-colors cursor-pointer"
                title="Copy Key Name"
              >
                <Copy class="w-3 h-3" />
              </button>
            </div>
          {/if}
        </div>

        <!-- Top Right Actions -->
        <div class="flex items-center gap-1.5 shrink-0">
          <button
            type="button"
            onclick={() => redisStore.refreshSelectedKey(connectionId)}
            class="flex items-center gap-1 px-2.5 py-1 text-xs bg-surface-900 hover:bg-surface-800 dark:bg-zinc-800/80 dark:hover:bg-zinc-700 text-slate-700 dark:text-zinc-300 rounded border border-slate-200 dark:border-zinc-700 transition-colors cursor-pointer shadow-xs"
            title="Reload Key Data"
          >
            <RefreshCw class="w-3 h-3" />
            <span>Reload</span>
          </button>
          <button
            type="button"
            onclick={() => onDeleteKey(detail.key)}
            class="flex items-center gap-1 px-2.5 py-1 text-xs bg-rose-50 hover:bg-rose-100 dark:bg-rose-950/40 dark:hover:bg-rose-900/60 text-rose-700 dark:text-rose-300 rounded border border-rose-200 dark:border-rose-900/40 transition-colors cursor-pointer shadow-xs"
            title="Delete this key"
          >
            <Trash2 class="w-3 h-3" />
            <span>Delete</span>
          </button>
        </div>
      </div>

      <!-- Metadata Badges: TTL & Memory Usage -->
      <div class="flex flex-wrap items-center gap-2 text-xs">
        <!-- TTL Editor Badge -->
        <div class="flex items-center gap-1.5 px-2 py-0.5 rounded bg-surface-950 dark:bg-zinc-950 border border-slate-200 dark:border-zinc-800">
          <Clock class="w-3 h-3 text-amber-500 dark:text-amber-400" />
          <span class="text-slate-500 dark:text-zinc-400 text-[11px]">TTL:</span>
          {#if isEditingTtl}
            <div class="flex items-center gap-1">
              <input
                type="number"
                bind:value={ttlInput}
                placeholder="-1 for persist"
                class="w-20 bg-surface-900 dark:bg-zinc-900 border border-indigo-500 rounded px-1.5 py-0.5 text-xs font-mono text-slate-900 dark:text-zinc-100 focus:outline-none"
              />
              <span class="text-[10px] text-slate-500 dark:text-zinc-500">sec</span>
              <button
                type="button"
                onclick={handleTtlSubmit}
                class="p-0.5 bg-indigo-600 hover:bg-indigo-500 text-white rounded text-[10px] cursor-pointer"
                title="Save TTL"
              >
                <Check class="w-3 h-3" />
              </button>
              <button
                type="button"
                onclick={() => (isEditingTtl = false)}
                class="p-0.5 bg-surface-800 dark:bg-zinc-800 text-slate-600 dark:text-zinc-400 rounded text-[10px] cursor-pointer"
                title="Cancel"
              >
                <X class="w-3 h-3" />
              </button>
            </div>
          {:else}
            <button
              type="button"
              onclick={() => { isEditingTtl = true; ttlInput = detail.ttl; }}
              class="font-mono font-medium hover:underline text-[11px] {detail.ttl === -1 ? 'text-emerald-600 dark:text-emerald-400' : 'text-amber-600 dark:text-amber-300'}"
              title="Click to edit TTL"
            >
              {detail.ttl === -1 ? 'Persistent (No Expiry)' : `${detail.ttl} seconds remaining`}
            </button>
          {/if}
        </div>

        <!-- Memory Usage Badge -->
        {#if detail.memoryUsageBytes !== undefined && detail.memoryUsageBytes !== null}
          <div class="flex items-center gap-1.5 px-2 py-0.5 rounded bg-surface-950 dark:bg-zinc-950 border border-slate-200 dark:border-zinc-800 text-[11px] text-slate-500 dark:text-zinc-400">
            <Cpu class="w-3 h-3 text-cyan-600 dark:text-cyan-400" />
            <span>Memory:</span>
            <span class="font-mono text-slate-700 dark:text-zinc-200 font-semibold">
              {detail.memoryUsageBytes > 1024 ? `${(detail.memoryUsageBytes / 1024).toFixed(1)} KB` : `${detail.memoryUsageBytes} Bytes`}
            </span>
          </div>
        {/if}
      </div>
    </div>

    <!-- Data Structure Inspector Body -->
    <div class="flex-1 overflow-y-auto p-3.5 custom-scrollbar">
      <!-- 1. STRING TYPE INSPECTOR -->
      {#if detail.keyType === 'string'}
        <div class="flex flex-col h-full gap-2">
          <!-- View Options / Toolbar -->
          <div class="flex items-center justify-between gap-2 pb-1">
            <div class="flex items-center gap-1 bg-surface-900 dark:bg-zinc-900 p-0.5 rounded border border-slate-200 dark:border-zinc-800">
              <button
                type="button"
                onclick={() => (stringViewMode = 'raw')}
                class="px-2 py-0.5 text-xs rounded transition-colors cursor-pointer {stringViewMode === 'raw' ? 'bg-surface-800 dark:bg-zinc-800 text-indigo-700 dark:text-indigo-300 font-medium shadow-xs' : 'text-slate-600 dark:text-zinc-400 hover:text-slate-900 dark:hover:text-zinc-200'}"
              >
                Raw Text
              </button>
              <button
                type="button"
                onclick={() => (stringViewMode = 'json')}
                class="px-2 py-0.5 text-xs rounded transition-colors cursor-pointer {stringViewMode === 'json' ? 'bg-surface-800 dark:bg-zinc-800 text-indigo-700 dark:text-indigo-300 font-medium shadow-xs' : 'text-slate-600 dark:text-zinc-400 hover:text-slate-900 dark:hover:text-zinc-200'}"
              >
                JSON
              </button>
            </div>

            <div class="flex items-center gap-2">
              {#if stringViewMode === 'json'}
                <button
                  type="button"
                  onclick={handleFormatJson}
                  class="flex items-center gap-1 px-2 py-1 text-xs bg-surface-900 hover:bg-surface-800 dark:bg-zinc-800 dark:hover:bg-zinc-700 text-slate-700 dark:text-zinc-200 rounded border border-slate-200 dark:border-zinc-700 transition-colors cursor-pointer shadow-xs"
                  title="Format JSON Pretty Print"
                >
                  <FileJson class="w-3.5 h-3.5 text-amber-500 dark:text-amber-400" />
                  <span>Beautify</span>
                </button>
              {/if}
              <button
                type="button"
                onclick={() => handleCopyValue(stringContent)}
                class="flex items-center gap-1 px-2 py-1 text-xs bg-surface-900 hover:bg-surface-800 dark:bg-zinc-800 dark:hover:bg-zinc-700 text-slate-700 dark:text-zinc-200 rounded border border-slate-200 dark:border-zinc-700 transition-colors cursor-pointer shadow-xs"
              >
                <Copy class="w-3.5 h-3.5" />
                <span>{copyFeedback ? 'Copied!' : 'Copy'}</span>
              </button>
              <button
                type="button"
                onclick={handleSaveString}
                disabled={!isStringDirty}
                class="flex items-center gap-1 px-3 py-1 text-xs bg-indigo-600 hover:bg-indigo-500 disabled:opacity-40 disabled:hover:bg-indigo-600 text-white rounded font-medium transition-colors shadow-xs cursor-pointer"
              >
                <Save class="w-3.5 h-3.5" />
                <span>Save</span>
              </button>
            </div>
          </div>

          <!-- Code / Text Area Editor -->
          <div class="flex-1 relative flex flex-col bg-surface-900 dark:bg-zinc-900 border border-slate-200 dark:border-zinc-800 rounded-lg overflow-hidden focus-within:border-indigo-500/60 shadow-xs">
            <textarea
              bind:value={stringContent}
              oninput={() => (isStringDirty = true)}
              class="w-full h-full bg-transparent p-3 text-xs font-mono text-slate-900 dark:text-zinc-200 resize-none focus:outline-none custom-scrollbar leading-relaxed"
              placeholder="String value..."
              spellcheck="false"
            ></textarea>
            <div class="p-1.5 bg-surface-950/80 dark:bg-zinc-950/80 border-t border-slate-200 dark:border-zinc-800/80 flex items-center justify-between text-[11px] text-slate-500 dark:text-zinc-500 font-mono">
              <span>{stringContent.length} characters • {new Blob([stringContent]).size} bytes</span>
              {#if isStringDirty}
                <span class="text-amber-500 dark:text-amber-400 font-medium">● Unsaved changes</span>
              {/if}
            </div>
          </div>
        </div>

      <!-- 2. HASH TYPE INSPECTOR -->
      {:else if detail.keyType === 'hash' && detail.valueHash}
        {@const entries = Object.entries(detail.valueHash).filter(([k, v]) =>
          !hashFilter || k.toLowerCase().includes(hashFilter.toLowerCase()) || v.toLowerCase().includes(hashFilter.toLowerCase())
        )}

        <div class="flex flex-col gap-3">
          <!-- Hash Controls & Search -->
          <div class="flex items-center justify-between gap-2">
            <div class="relative flex-1 max-w-sm">
              <Search class="w-3.5 h-3.5 absolute left-2.5 top-2.5 text-slate-400 dark:text-zinc-500 pointer-events-none" />
              <input
                type="text"
                bind:value={hashFilter}
                placeholder="Filter fields or values..."
                class="w-full bg-surface-900 dark:bg-zinc-900 border border-slate-200 dark:border-zinc-800 rounded pl-8 pr-3 py-1 text-xs text-slate-900 dark:text-zinc-200 placeholder-slate-400 dark:placeholder-zinc-500 focus:outline-none focus:border-indigo-500 font-mono shadow-xs"
              />
            </div>

            <button
              type="button"
              onclick={() => (isAddingHashField = true)}
              class="flex items-center gap-1 px-2.5 py-1 text-xs bg-indigo-600 hover:bg-indigo-500 text-white rounded font-medium transition-colors cursor-pointer shadow-xs"
            >
              <Plus class="w-3.5 h-3.5" />
              <span>Add Field</span>
            </button>
          </div>

          <!-- Add New Field Form -->
          {#if isAddingHashField}
            <div class="p-3 bg-surface-900 dark:bg-zinc-900/90 border border-indigo-300 dark:border-indigo-500/40 rounded-lg flex flex-col gap-2 shadow-xs">
              <span class="text-xs font-semibold text-indigo-700 dark:text-indigo-300">Add New Hash Field</span>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
                <input
                  type="text"
                  bind:value={newHashField}
                  placeholder="Field name (key)"
                  class="bg-surface-950 dark:bg-zinc-950 border border-slate-200 dark:border-zinc-800 rounded px-2.5 py-1 text-xs font-mono text-slate-900 dark:text-zinc-200 focus:outline-none focus:border-indigo-500"
                />
                <input
                  type="text"
                  bind:value={newHashValue}
                  placeholder="Field value"
                  class="bg-surface-950 dark:bg-zinc-950 border border-slate-200 dark:border-zinc-800 rounded px-2.5 py-1 text-xs font-mono text-slate-900 dark:text-zinc-200 focus:outline-none focus:border-indigo-500"
                />
              </div>
              <div class="flex items-center justify-end gap-2 pt-1">
                <button
                  type="button"
                  onclick={() => (isAddingHashField = false)}
                  class="px-2.5 py-1 text-xs bg-surface-800 dark:bg-zinc-800 hover:bg-surface-700 dark:hover:bg-zinc-700 text-slate-700 dark:text-zinc-300 rounded cursor-pointer"
                >
                  Cancel
                </button>
                <button
                  type="button"
                  onclick={handleAddHashField}
                  class="px-3 py-1 text-xs bg-indigo-600 hover:bg-indigo-500 text-white font-medium rounded cursor-pointer"
                >
                  Save Field
                </button>
              </div>
            </div>
          {/if}

          <!-- Hash Table -->
          <div class="border border-slate-200 dark:border-zinc-800 rounded-lg overflow-hidden bg-surface-900/40 dark:bg-zinc-900/40 shadow-xs">
            <table class="w-full text-left text-xs border-collapse">
              <thead>
                <tr class="bg-surface-900 dark:bg-zinc-900/80 border-b border-slate-200 dark:border-zinc-800 text-slate-600 dark:text-zinc-400 font-mono text-[11px]">
                  <th class="p-2.5 font-medium w-1/3">Field</th>
                  <th class="p-2.5 font-medium">Value</th>
                  <th class="p-2.5 font-medium text-right w-24">Actions</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-slate-200 dark:divide-zinc-800/60 font-mono">
                {#if entries.length === 0}
                  <tr>
                    <td colspan="3" class="p-6 text-center text-slate-500 dark:text-zinc-500">No fields matching filter.</td>
                  </tr>
                {:else}
                  {#each entries as [field, val] (field)}
                    <tr class="hover:bg-surface-800/50 dark:hover:bg-zinc-800/30 transition-colors group">
                      <td class="p-2.5 text-indigo-700 dark:text-indigo-300 font-semibold align-top">{field}</td>
                      <td class="p-2.5 text-slate-800 dark:text-zinc-200 align-top break-all">
                        {#if editingHashField === field}
                          <div class="flex items-center gap-1.5">
                            <input
                              type="text"
                              bind:value={editingHashValue}
                              class="w-full bg-surface-950 dark:bg-zinc-950 border border-indigo-500 rounded px-2 py-0.5 text-xs text-slate-900 dark:text-zinc-100 font-mono focus:outline-none"
                            />
                            <button
                              type="button"
                              onclick={handleSaveHashFieldEdit}
                              class="p-1 bg-indigo-600 hover:bg-indigo-500 text-white rounded cursor-pointer"
                            >
                              <Check class="w-3.5 h-3.5" />
                            </button>
                            <button
                              type="button"
                              onclick={() => (editingHashField = null)}
                              class="p-1 bg-surface-800 dark:bg-zinc-800 text-slate-600 dark:text-zinc-400 rounded cursor-pointer"
                            >
                              <X class="w-3.5 h-3.5" />
                            </button>
                          </div>
                        {:else}
                          <span class="select-all">{val}</span>
                        {/if}
                      </td>
                      <td class="p-2.5 text-right align-top">
                        <div class="flex items-center justify-end gap-1 opacity-80 group-hover:opacity-100">
                          <button
                            type="button"
                            onclick={() => { editingHashField = field; editingHashValue = val; }}
                            class="p-1 text-slate-400 hover:text-slate-700 dark:text-zinc-400 dark:hover:text-zinc-200 hover:bg-surface-800 dark:hover:bg-zinc-800 rounded cursor-pointer"
                            title="Edit Value"
                          >
                            <Edit2 class="w-3 h-3" />
                          </button>
                          <button
                            type="button"
                            onclick={() => handleDeleteHashField(field)}
                            class="p-1 text-rose-500 hover:text-rose-700 dark:text-rose-400/80 dark:hover:text-rose-300 hover:bg-rose-50 dark:hover:bg-rose-950/40 rounded cursor-pointer"
                            title="Delete Field"
                          >
                            <Trash2 class="w-3 h-3" />
                          </button>
                        </div>
                      </td>
                    </tr>
                  {/each}
                {/if}
              </tbody>
            </table>
          </div>
        </div>

      <!-- 3. LIST TYPE INSPECTOR -->
      {:else if detail.keyType === 'list' && detail.valueList}
        <div class="flex flex-col gap-3">
          <!-- List Controls -->
          <div class="flex items-center justify-between gap-2">
            <span class="text-xs text-slate-500 dark:text-zinc-400 font-mono">Length: <strong class="text-slate-800 dark:text-zinc-200">{detail.valueList.length}</strong> items</span>
            <button
              type="button"
              onclick={() => (isAddingListItem = true)}
              class="flex items-center gap-1 px-2.5 py-1 text-xs bg-emerald-600 hover:bg-emerald-500 text-white rounded font-medium transition-colors cursor-pointer shadow-xs"
            >
              <Plus class="w-3.5 h-3.5" />
              <span>Push Item</span>
            </button>
          </div>

          <!-- Add List Item Form -->
          {#if isAddingListItem}
            <div class="p-3 bg-surface-900 dark:bg-zinc-900/90 border border-emerald-300 dark:border-emerald-500/40 rounded-lg flex flex-col gap-2 shadow-xs">
              <div class="flex items-center justify-between">
                <span class="text-xs font-semibold text-emerald-700 dark:text-emerald-300">Push Element to List</span>
                <div class="flex items-center gap-2 text-xs">
                  <label class="flex items-center gap-1 cursor-pointer text-slate-700 dark:text-zinc-300">
                    <input type="radio" bind:group={listPushPos} value="left" />
                    <span>LPUSH (Head)</span>
                  </label>
                  <label class="flex items-center gap-1 cursor-pointer text-slate-700 dark:text-zinc-300">
                    <input type="radio" bind:group={listPushPos} value="right" />
                    <span>RPUSH (Tail)</span>
                  </label>
                </div>
              </div>
              <textarea
                bind:value={newListItem}
                placeholder="Element value..."
                class="bg-surface-950 dark:bg-zinc-950 border border-slate-200 dark:border-zinc-800 rounded p-2 text-xs font-mono text-slate-900 dark:text-zinc-200 resize-none h-16 focus:outline-none focus:border-emerald-500"
              ></textarea>
              <div class="flex items-center justify-end gap-2">
                <button
                  type="button"
                  onclick={() => (isAddingListItem = false)}
                  class="px-2.5 py-1 text-xs bg-surface-800 dark:bg-zinc-800 hover:bg-surface-700 dark:hover:bg-zinc-700 text-slate-700 dark:text-zinc-300 rounded cursor-pointer"
                >
                  Cancel
                </button>
                <button
                  type="button"
                  onclick={handleAddListItem}
                  class="px-3 py-1 text-xs bg-emerald-600 hover:bg-emerald-500 text-white font-medium rounded cursor-pointer"
                >
                  Push
                </button>
              </div>
            </div>
          {/if}

          <!-- List Elements Table -->
          <div class="border border-slate-200 dark:border-zinc-800 rounded-lg overflow-hidden bg-surface-900/40 dark:bg-zinc-900/40 shadow-xs">
            <table class="w-full text-left text-xs border-collapse">
              <thead>
                <tr class="bg-surface-900 dark:bg-zinc-900/80 border-b border-slate-200 dark:border-zinc-800 text-slate-600 dark:text-zinc-400 font-mono text-[11px]">
                  <th class="p-2.5 font-medium w-16">Index</th>
                  <th class="p-2.5 font-medium">Element Value</th>
                  <th class="p-2.5 font-medium text-right w-20">Action</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-slate-200 dark:divide-zinc-800/60 font-mono">
                {#if detail.valueList.length === 0}
                  <tr>
                    <td colspan="3" class="p-6 text-center text-slate-500 dark:text-zinc-500">List is empty.</td>
                  </tr>
                {:else}
                  {#each detail.valueList as item, index (index)}
                    <tr class="hover:bg-surface-800/50 dark:hover:bg-zinc-800/30 transition-colors group">
                      <td class="p-2.5 text-slate-400 dark:text-zinc-500 font-mono align-top">[{index}]</td>
                      <td class="p-2.5 text-slate-800 dark:text-zinc-200 align-top break-all select-all font-mono">{item}</td>
                      <td class="p-2.5 text-right align-top">
                        <button
                          type="button"
                          onclick={() => handleRemoveListItem(item)}
                          class="p-1 text-rose-500 hover:text-rose-700 dark:text-rose-400/80 dark:hover:text-rose-300 hover:bg-rose-50 dark:hover:bg-rose-950/40 rounded cursor-pointer"
                          title="Remove Element (LREM)"
                        >
                          <Trash2 class="w-3 h-3" />
                        </button>
                      </td>
                    </tr>
                  {/each}
                {/if}
              </tbody>
            </table>
          </div>
        </div>

      <!-- 4. SET TYPE INSPECTOR -->
      {:else if detail.keyType === 'set' && detail.valueSet}
        {@const filteredMembers = detail.valueSet.filter(m => !setFilter || m.toLowerCase().includes(setFilter.toLowerCase()))}

        <div class="flex flex-col gap-3">
          <div class="flex items-center justify-between gap-2">
            <div class="relative flex-1 max-w-sm">
              <Search class="w-3.5 h-3.5 absolute left-2.5 top-2.5 text-slate-400 dark:text-zinc-500 pointer-events-none" />
              <input
                type="text"
                bind:value={setFilter}
                placeholder="Filter set members..."
                class="w-full bg-surface-900 dark:bg-zinc-900 border border-slate-200 dark:border-zinc-800 rounded pl-8 pr-3 py-1 text-xs text-slate-900 dark:text-zinc-200 placeholder-slate-400 dark:placeholder-zinc-500 focus:outline-none focus:border-amber-500 font-mono shadow-xs"
              />
            </div>
            <button
              type="button"
              onclick={() => (isAddingSetMember = true)}
              class="flex items-center gap-1 px-2.5 py-1 text-xs bg-amber-600 hover:bg-amber-500 text-white rounded font-medium transition-colors cursor-pointer shadow-xs"
            >
              <Plus class="w-3.5 h-3.5" />
              <span>Add Member</span>
            </button>
          </div>

          {#if isAddingSetMember}
            <div class="p-3 bg-surface-900 dark:bg-zinc-900/90 border border-amber-300 dark:border-amber-500/40 rounded-lg flex flex-col gap-2 shadow-xs">
              <span class="text-xs font-semibold text-amber-800 dark:text-amber-300">Add Member to Set (SADD)</span>
              <input
                type="text"
                bind:value={newSetMember}
                placeholder="Member value..."
                class="bg-surface-950 dark:bg-zinc-950 border border-slate-200 dark:border-zinc-800 rounded px-2.5 py-1 text-xs font-mono text-slate-900 dark:text-zinc-200 focus:outline-none focus:border-amber-500"
              />
              <div class="flex items-center justify-end gap-2">
                <button
                  type="button"
                  onclick={() => (isAddingSetMember = false)}
                  class="px-2.5 py-1 text-xs bg-surface-800 dark:bg-zinc-800 hover:bg-surface-700 dark:hover:bg-zinc-700 text-slate-700 dark:text-zinc-300 rounded cursor-pointer"
                >
                  Cancel
                </button>
                <button
                  type="button"
                  onclick={handleAddSetMember}
                  class="px-3 py-1 text-xs bg-amber-600 hover:bg-amber-500 text-white font-medium rounded cursor-pointer"
                >
                  Add (SADD)
                </button>
              </div>
            </div>
          {/if}

          <!-- Set Members Grid / List -->
          <div class="border border-slate-200 dark:border-zinc-800 rounded-lg overflow-hidden bg-surface-900/40 dark:bg-zinc-900/40 shadow-xs">
            <table class="w-full text-left text-xs border-collapse">
              <thead>
                <tr class="bg-surface-900 dark:bg-zinc-900/80 border-b border-slate-200 dark:border-zinc-800 text-slate-600 dark:text-zinc-400 font-mono text-[11px]">
                  <th class="p-2.5 font-medium">Member Value</th>
                  <th class="p-2.5 font-medium text-right w-20">Action</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-slate-200 dark:divide-zinc-800/60 font-mono">
                {#if filteredMembers.length === 0}
                  <tr>
                    <td colspan="2" class="p-6 text-center text-slate-500 dark:text-zinc-500">No members found.</td>
                  </tr>
                {:else}
                  {#each filteredMembers as member (member)}
                    <tr class="hover:bg-surface-800/50 dark:hover:bg-zinc-800/30 transition-colors group">
                      <td class="p-2.5 text-slate-800 dark:text-zinc-200 break-all select-all">{member}</td>
                      <td class="p-2.5 text-right">
                        <button
                          type="button"
                          onclick={() => handleRemoveSetMember(member)}
                          class="p-1 text-rose-500 hover:text-rose-700 dark:text-rose-400/80 dark:hover:text-rose-300 hover:bg-rose-50 dark:hover:bg-rose-950/40 rounded cursor-pointer"
                          title="Remove Member (SREM)"
                        >
                          <Trash2 class="w-3 h-3" />
                        </button>
                      </td>
                    </tr>
                  {/each}
                {/if}
              </tbody>
            </table>
          </div>
        </div>

      <!-- 5. ZSET (SORTED SET) TYPE INSPECTOR -->
      {:else if detail.keyType === 'zset' && detail.valueZset}
        {@const filteredZset = detail.valueZset.filter(m => !zsetFilter || m.member.toLowerCase().includes(zsetFilter.toLowerCase()))}

        <div class="flex flex-col gap-3">
          <div class="flex items-center justify-between gap-2">
            <div class="relative flex-1 max-w-sm">
              <Search class="w-3.5 h-3.5 absolute left-2.5 top-2.5 text-slate-400 dark:text-zinc-500 pointer-events-none" />
              <input
                type="text"
                bind:value={zsetFilter}
                placeholder="Filter members..."
                class="w-full bg-surface-900 dark:bg-zinc-900 border border-slate-200 dark:border-zinc-800 rounded pl-8 pr-3 py-1 text-xs text-slate-900 dark:text-zinc-200 placeholder-slate-400 dark:placeholder-zinc-500 focus:outline-none focus:border-rose-500 font-mono shadow-xs"
              />
            </div>
            <button
              type="button"
              onclick={() => (isAddingZSetMember = true)}
              class="flex items-center gap-1 px-2.5 py-1 text-xs bg-rose-600 hover:bg-rose-500 text-white rounded font-medium transition-colors cursor-pointer shadow-xs"
            >
              <Plus class="w-3.5 h-3.5" />
              <span>Add Member</span>
            </button>
          </div>

          {#if isAddingZSetMember}
            <div class="p-3 bg-surface-900 dark:bg-zinc-900/90 border border-rose-300 dark:border-rose-500/40 rounded-lg flex flex-col gap-2 shadow-xs">
              <span class="text-xs font-semibold text-rose-800 dark:text-rose-300">Add Member with Score (ZADD)</span>
              <div class="grid grid-cols-1 md:grid-cols-3 gap-2">
                <input
                  type="text"
                  bind:value={newZSetMember}
                  placeholder="Member name"
                  class="col-span-2 bg-surface-950 dark:bg-zinc-950 border border-slate-200 dark:border-zinc-800 rounded px-2.5 py-1 text-xs font-mono text-slate-900 dark:text-zinc-200 focus:outline-none focus:border-rose-500"
                />
                <input
                  type="number"
                  step="any"
                  bind:value={newZSetScore}
                  placeholder="Score (e.g. 100)"
                  class="bg-surface-950 dark:bg-zinc-950 border border-slate-200 dark:border-zinc-800 rounded px-2.5 py-1 text-xs font-mono text-slate-900 dark:text-zinc-200 focus:outline-none focus:border-rose-500"
                />
              </div>
              <div class="flex items-center justify-end gap-2 pt-1">
                <button
                  type="button"
                  onclick={() => (isAddingZSetMember = false)}
                  class="px-2.5 py-1 text-xs bg-surface-800 dark:bg-zinc-800 hover:bg-surface-700 dark:hover:bg-zinc-700 text-slate-700 dark:text-zinc-300 rounded cursor-pointer"
                >
                  Cancel
                </button>
                <button
                  type="button"
                  onclick={handleAddZSetMember}
                  class="px-3 py-1 text-xs bg-rose-600 hover:bg-rose-500 text-white font-medium rounded cursor-pointer"
                >
                  Add (ZADD)
                </button>
              </div>
            </div>
          {/if}

          <!-- Sorted Set Table -->
          <div class="border border-slate-200 dark:border-zinc-800 rounded-lg overflow-hidden bg-surface-900/40 dark:bg-zinc-900/40 shadow-xs">
            <table class="w-full text-left text-xs border-collapse">
              <thead>
                <tr class="bg-surface-900 dark:bg-zinc-900/80 border-b border-slate-200 dark:border-zinc-800 text-slate-600 dark:text-zinc-400 font-mono text-[11px]">
                  <th class="p-2.5 font-medium w-16">Rank</th>
                  <th class="p-2.5 font-medium w-28">Score</th>
                  <th class="p-2.5 font-medium">Member</th>
                  <th class="p-2.5 font-medium text-right w-20">Action</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-slate-200 dark:divide-zinc-800/60 font-mono">
                {#if filteredZset.length === 0}
                  <tr>
                    <td colspan="4" class="p-6 text-center text-slate-500 dark:text-zinc-500">No sorted set members found.</td>
                  </tr>
                {:else}
                  {#each filteredZset as item, rank (item.member)}
                    <tr class="hover:bg-surface-800/50 dark:hover:bg-zinc-800/30 transition-colors group">
                      <td class="p-2.5 text-slate-400 dark:text-zinc-500 font-mono align-top">#{rank + 1}</td>
                      <td class="p-2.5 text-rose-600 dark:text-rose-300 font-bold align-top">
                        {#if editingZSetMember === item.member}
                          <div class="flex items-center gap-1">
                            <input
                              type="number"
                              step="any"
                              bind:value={editingZSetScore}
                              class="w-20 bg-surface-950 dark:bg-zinc-950 border border-rose-500 rounded px-1.5 py-0.5 text-xs text-slate-900 dark:text-zinc-100 font-mono focus:outline-none"
                            />
                            <button
                              type="button"
                              onclick={handleSaveZSetScoreEdit}
                              class="p-0.5 bg-rose-600 text-white rounded cursor-pointer"
                            >
                              <Check class="w-3 h-3" />
                            </button>
                            <button
                              type="button"
                              onclick={() => (editingZSetMember = null)}
                              class="p-0.5 bg-surface-800 dark:bg-zinc-800 text-slate-600 dark:text-zinc-400 rounded cursor-pointer"
                            >
                              <X class="w-3 h-3" />
                            </button>
                          </div>
                        {:else}
                          <button
                            type="button"
                            onclick={() => { editingZSetMember = item.member; editingZSetScore = item.score; }}
                            class="hover:underline cursor-pointer"
                            title="Click to edit score"
                          >
                            {item.score}
                          </button>
                        {/if}
                      </td>
                      <td class="p-2.5 text-slate-800 dark:text-zinc-200 align-top break-all select-all">{item.member}</td>
                      <td class="p-2.5 text-right align-top">
                        <button
                          type="button"
                          onclick={() => handleRemoveZSetMember(item.member)}
                          class="p-1 text-rose-500 hover:text-rose-700 dark:text-rose-400/80 dark:hover:text-rose-300 hover:bg-rose-50 dark:hover:bg-rose-950/40 rounded cursor-pointer"
                          title="Remove Member (ZREM)"
                        >
                          <Trash2 class="w-3 h-3" />
                        </button>
                      </td>
                    </tr>
                  {/each}
                {/if}
              </tbody>
            </table>
          </div>
        </div>

      <!-- 6. STREAM TYPE INSPECTOR -->
      {:else if detail.keyType === 'stream' && detail.valueStream}
        <div class="flex flex-col gap-3">
          <div class="flex items-center justify-between">
            <span class="text-xs text-slate-500 dark:text-zinc-400 font-mono">Stream Entries (XREVRANGE latest {detail.valueStream.length}):</span>
          </div>

          <div class="space-y-2">
            {#if detail.valueStream.length === 0}
              <div class="p-8 border border-slate-200 dark:border-zinc-800 rounded-lg text-center text-slate-500 dark:text-zinc-500 text-xs">
                No entries in this stream.
              </div>
            {:else}
              {#each detail.valueStream as entry (entry.id)}
                <div class="p-3 bg-surface-900 dark:bg-zinc-900/60 border border-slate-200 dark:border-zinc-800 rounded-lg flex flex-col gap-2 font-mono shadow-xs">
                  <div class="flex items-center justify-between text-xs border-b border-slate-200 dark:border-zinc-800/60 pb-1.5">
                    <span class="text-cyan-700 dark:text-cyan-400 font-semibold">{entry.id}</span>
                    <span class="text-[11px] text-slate-500 dark:text-zinc-500">
                      {new Date(parseInt(entry.id.split('-')[0], 10) || Date.now()).toLocaleTimeString()}
                    </span>
                  </div>
                  <div class="grid grid-cols-1 md:grid-cols-2 gap-2 text-xs">
                    {#each Object.entries(entry.fields) as [k, v] (k)}
                      <div class="flex items-center gap-1.5 bg-surface-950/70 dark:bg-zinc-950/70 px-2 py-1 rounded border border-slate-200 dark:border-zinc-800/50">
                        <span class="text-slate-500 dark:text-zinc-400">{k}:</span>
                        <span class="text-slate-800 dark:text-zinc-100 font-medium truncate select-all">{v}</span>
                      </div>
                    {/each}
                  </div>
                </div>
              {/each}
            {/if}
          </div>
        </div>

      {:else}
        <div class="p-8 text-center text-slate-500 dark:text-zinc-500 text-xs">
          Unsupported or empty data structure.
        </div>
      {/if}
    </div>
  </div>
{/if}/if}
