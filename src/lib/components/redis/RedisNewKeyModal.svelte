<script lang="ts">
  import { X, Plus, Key, Clock, Database, Layers } from 'lucide-svelte';
  import { redisStore } from '../../state/redis.svelte';
  import type { RedisKeyType } from '../../api/types';
  import CustomSelect from '../ui/CustomSelect.svelte';

  let {
    connectionId,
    isOpen,
    onClose
  }: {
    connectionId: string;
    isOpen: boolean;
    onClose: () => void;
  } = $props();

  const redisState = $derived(redisStore.getState(connectionId));

  let keyName = $state('');
  let keyType = $state<RedisKeyType>('string');
  let stringValue = $state('');
  let hashField = $state('field1');
  let hashValue = $state('');
  let listItemValue = $state('');
  let setMemberValue = $state('');
  let zsetMemberValue = $state('');
  let zsetScore = $state<number>(1.0);
  let ttlSeconds = $state<string>('-1');
  let isSubmitting = $state(false);

  const typeOptions = [
    { value: 'string', label: 'String (Raw / JSON / Text)' },
    { value: 'hash', label: 'Hash (Field-Value Map)' },
    { value: 'list', label: 'List (Ordered Array)' },
    { value: 'set', label: 'Set (Unique Members)' },
    { value: 'zset', label: 'Sorted Set (Score-Ranked)' }
  ];

  async function handleCreate() {
    if (!keyName.trim()) {
      alert('Please provide a key name');
      return;
    }

    isSubmitting = true;
    try {
      const parsedTtl = parseInt(ttlSeconds, 10);
      const ttl = isNaN(parsedTtl) || parsedTtl <= 0 ? undefined : parsedTtl;

      let value = '';
      let field: string | undefined = undefined;
      let score: number | undefined = undefined;

      if (keyType === 'string') {
        value = stringValue;
      } else if (keyType === 'hash') {
        field = hashField.trim() || 'field1';
        value = hashValue;
      } else if (keyType === 'list') {
        value = listItemValue;
      } else if (keyType === 'set') {
        value = setMemberValue;
      } else if (keyType === 'zset') {
        value = zsetMemberValue;
        score = zsetScore;
      }

      await redisStore.createNewKey(connectionId, {
        key: keyName.trim(),
        keyType,
        value,
        ttl,
        field,
        score
      });

      // Reset form
      keyName = '';
      stringValue = '';
      hashField = 'field1';
      hashValue = '';
      listItemValue = '';
      setMemberValue = '';
      zsetMemberValue = '';
      zsetScore = 1.0;
      ttlSeconds = '-1';

      onClose();
    } catch (err: any) {
      alert(`Failed to create key: ${err?.message || err}`);
    } finally {
      isSubmitting = false;
    }
  }
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 dark:bg-black/80 backdrop-blur-xs p-4 animate-in fade-in duration-150">
    <div class="bg-surface-900 dark:bg-zinc-900 border border-slate-200 dark:border-zinc-700/80 rounded-2xl shadow-2xl w-full max-w-lg overflow-hidden flex flex-col max-h-[90vh]">
      <!-- Header -->
      <div class="p-4 border-b border-slate-200 dark:border-zinc-800 flex items-center justify-between bg-surface-900 dark:bg-zinc-900/90">
        <div class="flex items-center gap-2.5">
          <div class="p-2 bg-indigo-50 dark:bg-indigo-600/20 text-indigo-600 dark:text-indigo-400 rounded-lg border border-indigo-200 dark:border-indigo-500/30">
            <Plus class="w-4 h-4" />
          </div>
          <div>
            <h3 class="text-sm font-semibold text-slate-900 dark:text-zinc-100">Create New Redis Key</h3>
            <p class="text-[11px] text-slate-500 dark:text-zinc-400">Add a new key-value entry into DB {redisState.selectedDb}</p>
          </div>
        </div>

        <button
          type="button"
          onclick={onClose}
          class="p-1.5 text-slate-400 hover:text-slate-700 dark:text-zinc-400 dark:hover:text-zinc-200 hover:bg-surface-800 dark:hover:bg-zinc-800 rounded-md transition-colors cursor-pointer"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Body Form -->
      <div class="p-5 overflow-y-auto space-y-4 custom-scrollbar">
        <!-- Key Name -->
        <div class="space-y-1.5">
          <label class="text-xs font-medium text-slate-700 dark:text-zinc-300 flex items-center gap-1.5">
            <Key class="w-3.5 h-3.5 text-indigo-600 dark:text-indigo-400" />
            Key Name
          </label>
          <input
            type="text"
            bind:value={keyName}
            placeholder="e.g. user:1001:profile or cache:feed:trending"
            class="w-full bg-surface-950 dark:bg-zinc-950 border border-slate-200 dark:border-zinc-700 rounded-lg px-3 py-2 text-xs font-mono text-slate-900 dark:text-zinc-100 placeholder-slate-400 dark:placeholder-zinc-500 focus:outline-none focus:border-indigo-500 shadow-xs"
            autofocus
          />
          <p class="text-[10px] text-slate-500 dark:text-zinc-500">Tip: Use colons (:) to create hierarchical namespaces in the tree view.</p>
        </div>

        <!-- Data Structure Type -->
        <div class="space-y-1.5">
          <label class="text-xs font-medium text-slate-700 dark:text-zinc-300 flex items-center gap-1.5">
            <Layers class="w-3.5 h-3.5 text-indigo-600 dark:text-indigo-400" />
            Data Structure Type
          </label>
          <CustomSelect
            value={keyType}
            options={typeOptions}
            onchange={(val) => (keyType = val as RedisKeyType)}
          />
        </div>

        <!-- Type Specific Initial Value Inputs -->
        {#if keyType === 'string'}
          <div class="space-y-1.5">
            <label class="text-xs font-medium text-slate-700 dark:text-zinc-300">Initial String / JSON Value</label>
            <textarea
              bind:value={stringValue}
              placeholder="Enter text, number, or JSON string..."
              class="w-full bg-surface-950 dark:bg-zinc-950 border border-slate-200 dark:border-zinc-700 rounded-lg p-2.5 text-xs font-mono text-slate-900 dark:text-zinc-100 resize-none h-24 focus:outline-none focus:border-indigo-500 shadow-xs"
            ></textarea>
          </div>
        {:else if keyType === 'hash'}
          <div class="grid grid-cols-2 gap-3">
            <div class="space-y-1.5">
              <label class="text-xs font-medium text-slate-700 dark:text-zinc-300">Initial Field Name</label>
              <input
                type="text"
                bind:value={hashField}
                placeholder="field_name"
                class="w-full bg-surface-950 dark:bg-zinc-950 border border-slate-200 dark:border-zinc-700 rounded-lg px-3 py-2 text-xs font-mono text-slate-900 dark:text-zinc-100 focus:outline-none focus:border-indigo-500 shadow-xs"
              />
            </div>
            <div class="space-y-1.5">
              <label class="text-xs font-medium text-slate-700 dark:text-zinc-300">Field Value</label>
              <input
                type="text"
                bind:value={hashValue}
                placeholder="value"
                class="w-full bg-surface-950 dark:bg-zinc-950 border border-slate-200 dark:border-zinc-700 rounded-lg px-3 py-2 text-xs font-mono text-slate-900 dark:text-zinc-100 focus:outline-none focus:border-indigo-500 shadow-xs"
              />
            </div>
          </div>
        {:else if keyType === 'list'}
          <div class="space-y-1.5">
            <label class="text-xs font-medium text-slate-700 dark:text-zinc-300">Initial Element Value</label>
            <input
              type="text"
              bind:value={listItemValue}
              placeholder="Element string / JSON"
              class="w-full bg-surface-950 dark:bg-zinc-950 border border-slate-200 dark:border-zinc-700 rounded-lg px-3 py-2 text-xs font-mono text-slate-900 dark:text-zinc-100 focus:outline-none focus:border-indigo-500 shadow-xs"
            />
          </div>
        {:else if keyType === 'set'}
          <div class="space-y-1.5">
            <label class="text-xs font-medium text-slate-700 dark:text-zinc-300">Initial Set Member</label>
            <input
              type="text"
              bind:value={setMemberValue}
              placeholder="Member value"
              class="w-full bg-surface-950 dark:bg-zinc-950 border border-slate-200 dark:border-zinc-700 rounded-lg px-3 py-2 text-xs font-mono text-slate-900 dark:text-zinc-100 focus:outline-none focus:border-indigo-500 shadow-xs"
            />
          </div>
        {:else if keyType === 'zset'}
          <div class="grid grid-cols-3 gap-3">
            <div class="col-span-2 space-y-1.5">
              <label class="text-xs font-medium text-slate-700 dark:text-zinc-300">Initial Member</label>
              <input
                type="text"
                bind:value={zsetMemberValue}
                placeholder="Member name"
                class="w-full bg-surface-950 dark:bg-zinc-950 border border-slate-200 dark:border-zinc-700 rounded-lg px-3 py-2 text-xs font-mono text-slate-900 dark:text-zinc-100 focus:outline-none focus:border-indigo-500 shadow-xs"
              />
            </div>
            <div class="space-y-1.5">
              <label class="text-xs font-medium text-slate-700 dark:text-zinc-300">Score</label>
              <input
                type="number"
                step="any"
                bind:value={zsetScore}
                placeholder="1.0"
                class="w-full bg-surface-950 dark:bg-zinc-950 border border-slate-200 dark:border-zinc-700 rounded-lg px-3 py-2 text-xs font-mono text-slate-900 dark:text-zinc-100 focus:outline-none focus:border-indigo-500 shadow-xs"
              />
            </div>
          </div>
        {/if}

        <!-- Expiration TTL -->
        <div class="space-y-1.5">
          <label class="text-xs font-medium text-slate-700 dark:text-zinc-300 flex items-center justify-between">
            <span class="flex items-center gap-1.5">
              <Clock class="w-3.5 h-3.5 text-amber-500 dark:text-amber-400" />
              Time-To-Live (TTL)
            </span>
            <span class="text-[11px] text-slate-500 dark:text-zinc-500 font-mono">-1 = Persistent (No Expiry)</span>
          </label>
          <div class="flex items-center gap-2">
            <input
              type="number"
              bind:value={ttlSeconds}
              placeholder="-1"
              class="w-full bg-surface-950 dark:bg-zinc-950 border border-slate-200 dark:border-zinc-700 rounded-lg px-3 py-2 text-xs font-mono text-slate-900 dark:text-zinc-100 focus:outline-none focus:border-indigo-500 shadow-xs"
            />
            <span class="text-xs text-slate-500 dark:text-zinc-400 font-mono shrink-0">seconds</span>
          </div>
        </div>
      </div>

      <!-- Footer Buttons -->
      <div class="p-4 border-t border-slate-200 dark:border-zinc-800 bg-surface-900 dark:bg-zinc-900/90 flex items-center justify-end gap-2.5">
        <button
          type="button"
          onclick={onClose}
          class="px-3.5 py-1.5 text-xs text-slate-600 dark:text-zinc-400 hover:text-slate-900 dark:hover:text-zinc-200 hover:bg-surface-800 dark:hover:bg-zinc-800 rounded-lg transition-colors cursor-pointer"
        >
          Cancel
        </button>
        <button
          type="button"
          onclick={handleCreate}
          disabled={isSubmitting || !keyName.trim()}
          class="flex items-center gap-1.5 px-4 py-1.5 text-xs bg-indigo-600 hover:bg-indigo-500 disabled:opacity-40 disabled:hover:bg-indigo-600 text-white font-medium rounded-lg shadow-sm transition-colors cursor-pointer"
        >
          <Plus class="w-3.5 h-3.5" />
          <span>{isSubmitting ? 'Creating Key...' : 'Create Key'}</span>
        </button>
      </div>
    </div>
  </div>
{/if}
