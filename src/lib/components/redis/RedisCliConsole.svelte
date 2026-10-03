<script lang="ts">
  import { Terminal, Send, Trash2, Zap, Clock, CornerDownLeft, Sparkles } from 'lucide-svelte';
  import { redisStore } from '../../state/redis.svelte';
  import type { RedisCliResponse } from '../../api/types';

  let { connectionId }: { connectionId: string } = $props();

  const redisState = $derived(redisStore.getState(connectionId));

  let inputCommand = $state('');
  let historyIndex = $state<number>(-1);
  let commandHistory = $state<string[]>([]);
  let logsContainer = $state<HTMLDivElement | null>(null);
  let isExecuting = $state(false);

  // Quick command suggestions
  const quickCommands = [
    'PING',
    'INFO',
    'KEYS *',
    'DBSIZE',
    'CLIENT LIST',
    'SLOWLOG GET 10',
    'CONFIG GET maxmemory'
  ];

  function scrollToBottom() {
    setTimeout(() => {
      if (logsContainer) {
        logsContainer.scrollTop = logsContainer.scrollHeight;
      }
    }, 50);
  }

  async function handleSubmit() {
    const cmd = inputCommand.trim();
    if (!cmd || isExecuting) return;

    // Add to history
    commandHistory = [cmd, ...commandHistory.filter(c => c !== cmd)].slice(0, 50);
    historyIndex = -1;
    inputCommand = '';
    isExecuting = true;

    try {
      await redisStore.executeCli(connectionId, cmd);
      scrollToBottom();
    } finally {
      isExecuting = false;
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (commandHistory.length > 0 && historyIndex < commandHistory.length - 1) {
        historyIndex++;
        inputCommand = commandHistory[historyIndex];
      }
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (historyIndex > 0) {
        historyIndex--;
        inputCommand = commandHistory[historyIndex];
      } else if (historyIndex === 0) {
        historyIndex = -1;
        inputCommand = '';
      }
    }
  }

  function runQuickCommand(cmd: string) {
    inputCommand = cmd;
    handleSubmit();
  }

  function clearLogs() {
    redisState.cliLogs = [];
  }
</script>

<div class="h-full flex flex-col bg-zinc-950 text-zinc-200 font-mono select-text">
  <!-- CLI Header & Quick Actions -->
  <div class="p-2.5 bg-zinc-900/90 border-b border-zinc-800 flex items-center justify-between gap-3 shrink-0">
    <div class="flex items-center gap-2">
      <Terminal class="w-4 h-4 text-emerald-400" />
      <span class="text-xs font-semibold text-zinc-200">Interactive Redis CLI Console</span>
      <span class="text-[10px] px-1.5 py-0.5 rounded bg-emerald-950/60 text-emerald-400 border border-emerald-800/40">
        DB {redisState.selectedDb}
      </span>
    </div>

    <!-- Quick Shortcuts Bar -->
    <div class="hidden lg:flex items-center gap-1.5 overflow-x-auto">
      <span class="text-[10px] text-zinc-500 font-sans">Quick:</span>
      {#each quickCommands as cmd}
        <button
          type="button"
          onclick={() => runQuickCommand(cmd)}
          class="px-2 py-0.5 text-[10px] bg-zinc-800 hover:bg-zinc-700 text-zinc-300 rounded border border-zinc-700/60 transition-colors cursor-pointer"
        >
          {cmd}
        </button>
      {/each}
    </div>

    <button
      type="button"
      onclick={clearLogs}
      class="flex items-center gap-1 px-2 py-1 text-xs text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800 rounded transition-colors cursor-pointer"
      title="Clear Console Output"
    >
      <Trash2 class="w-3 h-3" />
      <span class="text-[11px] font-sans">Clear</span>
    </button>
  </div>

  <!-- CLI Output Logs Area -->
  <div
    bind:this={logsContainer}
    class="flex-1 overflow-y-auto p-3 space-y-3 custom-scrollbar text-xs leading-relaxed"
  >
    {#each redisState.cliLogs as log, i (i)}
      <div class="space-y-1">
        <!-- Command Sent -->
        <div class="flex items-center gap-2 text-zinc-400">
          <span class="text-emerald-400 font-bold">127.0.0.1:6379[{redisState.selectedDb}]&gt;</span>
          <span class="text-zinc-100 font-semibold">{log.command}</span>
          {#if log.durationMs > 0}
            <span class="text-[10px] text-zinc-600 ml-auto flex items-center gap-0.5">
              <Clock class="w-2.5 h-2.5" />
              {log.durationMs.toFixed(2)}ms
            </span>
          {/if}
        </div>

        <!-- Response Output -->
        <div class="pl-4 border-l-2 {log.responseType === 'error' ? 'border-rose-500/60 text-rose-300 bg-rose-950/20' : 'border-zinc-800 text-zinc-300 bg-zinc-900/30'} p-2 rounded-r whitespace-pre-wrap font-mono">
          {log.response}
        </div>
      </div>
    {/each}
  </div>

  <!-- CLI Input Command Bar -->
  <div class="p-2.5 bg-zinc-900/90 border-t border-zinc-800 shrink-0">
    <form
      onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}
      class="flex items-center gap-2 bg-zinc-950 border border-zinc-700/80 rounded-lg px-3 py-1.5 focus-within:border-emerald-500 transition-colors shadow-inner"
    >
      <span class="text-emerald-400 font-bold shrink-0 text-xs select-none">
        redis[{redisState.selectedDb}]&gt;
      </span>
      <input
        type="text"
        bind:value={inputCommand}
        onkeydown={handleKeyDown}
        placeholder="Type a Redis command (e.g. GET my_key, HGETALL user:101, INFO) - Up/Down for history"
        class="w-full bg-transparent text-xs text-zinc-100 placeholder-zinc-600 focus:outline-none font-mono"
        disabled={isExecuting}
        autofocus
      />
      <button
        type="submit"
        disabled={!inputCommand.trim() || isExecuting}
        class="flex items-center gap-1 px-2.5 py-1 text-xs bg-emerald-600 hover:bg-emerald-500 disabled:opacity-40 disabled:hover:bg-emerald-600 text-white rounded font-sans font-medium transition-colors shrink-0 cursor-pointer"
      >
        <Send class="w-3 h-3" />
        <span>Run</span>
      </button>
    </form>
  </div>
</div>
