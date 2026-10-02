<script lang="ts">
  import { 
    Clock, 
    Calendar, 
    Globe, 
    Copy, 
    Check, 
    RotateCcw, 
    ArrowRight,
    Sparkles,
    CalendarCheck
  } from 'lucide-svelte';

  let { 
    value, 
    onChange 
  }: { 
    value: any; 
    onChange: (newVal: any) => void; 
  } = $props();

  let parsedDate = $state<Date | null>(null);
  let rawInput = $state<string>('');
  let copiedKey = $state<string | null>(null);
  let selectedTz = $state<string>('UTC');

  const TIMEZONES = [
    { label: 'UTC (Coordinated Universal Time)', tz: 'UTC' },
    { label: 'Local System Time', tz: Intl.DateTimeFormat().resolvedOptions().timeZone },
    { label: 'Jakarta / Bangkok (WIB, UTC+7)', tz: 'Asia/Jakarta' },
    { label: 'Singapore / Kuala Lumpur (SGT, UTC+8)', tz: 'Asia/Singapore' },
    { label: 'Tokyo / Seoul (JST, UTC+9)', tz: 'Asia/Tokyo' },
    { label: 'London (GMT / BST)', tz: 'Europe/London' },
    { label: 'New York (EST / EDT, UTC-5)', tz: 'America/New_York' },
    { label: 'Los Angeles (PST / PDT, UTC-8)', tz: 'America/Los_Angeles' },
    { label: 'Sydney (AEST, UTC+10)', tz: 'Australia/Sydney' },
  ];

  $effect(() => {
    rawInput = String(value ?? '');
    parseDateInput(rawInput);
  });

  function parseDateInput(str: string) {
    if (!str.trim()) {
      parsedDate = null;
      return;
    }

    // Check if numeric unix timestamp
    const num = Number(str);
    if (!isNaN(num)) {
      if (str.length === 10) {
        parsedDate = new Date(num * 1000);
        return;
      } else if (str.length === 13) {
        parsedDate = new Date(num);
        return;
      }
    }

    const d = new Date(str);
    if (!isNaN(d.getTime())) {
      parsedDate = d;
    } else {
      parsedDate = null;
    }
  }

  function getRelativeTime(d: Date): string {
    const diff = Date.now() - d.getTime();
    const absDiff = Math.abs(diff);
    const isPast = diff > 0;

    const prefix = isPast ? '' : 'in ';
    const suffix = isPast ? ' ago' : '';

    if (absDiff < 60_000) return 'Just now';
    if (absDiff < 3600_000) return `${prefix}${Math.floor(absDiff / 60_000)} minutes${suffix}`;
    if (absDiff < 86400_000) return `${prefix}${Math.floor(absDiff / 3600_000)} hours${suffix}`;
    if (absDiff < 2592000_000) return `${prefix}${Math.floor(absDiff / 86400_000)} days${suffix}`;
    if (absDiff < 31536000_000) return `${prefix}${Math.floor(absDiff / 2592000_000)} months${suffix}`;
    return `${prefix}${Math.floor(absDiff / 31536000_000)} years${suffix}`;
  }

  function formatForTimezone(d: Date, tz: string): string {
    try {
      return new Intl.DateTimeFormat('en-US', {
        timeZone: tz,
        year: 'numeric',
        month: 'short',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
        hour12: false,
        timeZoneName: 'short'
      }).format(d);
    } catch {
      return d.toISOString();
    }
  }

  function copyText(key: string, text: string) {
    navigator.clipboard.writeText(text);
    copiedKey = key;
    setTimeout(() => copiedKey = null, 1500);
  }

  function setToNow() {
    const now = new Date();
    parsedDate = now;
    rawInput = now.toISOString();
    onChange(now.toISOString());
  }

  function handleDateTimeEdit(isoString: string) {
    parseDateInput(isoString);
    if (parsedDate) {
      onChange(isoString);
    }
  }
</script>

<div class="flex-1 overflow-auto p-4 space-y-4 bg-surface-950 text-xs">
  {#if !parsedDate}
    <div class="p-6 bg-surface-900 border border-slate-200 dark:border-slate-800 rounded-xl text-center space-y-3">
      <Clock size={32} class="mx-auto text-slate-500 stroke-1" />
      <p class="text-sm font-semibold text-slate-300">Invalid Date or Timestamp Format</p>
      <p class="text-xs text-slate-500 max-w-sm mx-auto font-mono">
        Current value: "{rawInput}" could not be parsed as a valid ISO-8601 or Epoch timestamp.
      </p>
      <button
        type="button"
        onclick={setToNow}
        class="px-3 py-1.5 bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg text-xs font-semibold"
      >
        Set to Current Timestamp (NOW)
      </button>
    </div>
  {:else}
    <!-- Relative Time Banner -->
    <div class="p-3 bg-indigo-500/10 border border-indigo-500/20 rounded-xl flex items-center justify-between">
      <div class="flex items-center gap-2">
        <Clock size={16} class="text-indigo-400" />
        <div>
          <span class="font-bold text-slate-900 dark:text-white text-xs">{getRelativeTime(parsedDate)}</span>
          <span class="text-[11px] text-slate-500 dark:text-slate-400 font-mono ml-2">
            ({parsedDate.toUTCString()})
          </span>
        </div>
      </div>
      <button
        type="button"
        onclick={setToNow}
        class="px-2 py-1 text-[11px] bg-surface-900 hover:bg-surface-800 text-indigo-400 border border-indigo-500/30 rounded font-semibold cursor-pointer"
      >
        Set to NOW
      </button>
    </div>

    <!-- Conversions Grid -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
      <!-- 1. UTC ISO-8601 -->
      <div class="bg-surface-900 border border-slate-200 dark:border-slate-800 rounded-xl p-3 flex flex-col justify-between gap-1.5">
        <div class="flex items-center justify-between text-slate-500 dark:text-slate-400 text-[11px]">
          <span class="font-bold">ISO-8601 (UTC)</span>
          <button
            type="button"
            onclick={() => copyText('iso', parsedDate!.toISOString())}
            class="hover:text-slate-200 p-0.5"
          >
            {#if copiedKey === 'iso'}
              <Check size={12} class="text-emerald-500" />
            {:else}
              <Copy size={12} />
            {/if}
          </button>
        </div>
        <div class="font-mono text-xs text-indigo-600 dark:text-indigo-300 select-all">
          {parsedDate.toISOString()}
        </div>
        <button
          type="button"
          onclick={() => onChange(parsedDate!.toISOString())}
          class="text-[10px] text-slate-500 hover:text-indigo-400 text-left cursor-pointer"
        >
          Use as cell value
        </button>
      </div>

      <!-- 2. Local System Time -->
      <div class="bg-surface-900 border border-slate-200 dark:border-slate-800 rounded-xl p-3 flex flex-col justify-between gap-1.5">
        <div class="flex items-center justify-between text-slate-500 dark:text-slate-400 text-[11px]">
          <span class="font-bold">Local Time ({Intl.DateTimeFormat().resolvedOptions().timeZone})</span>
          <button
            type="button"
            onclick={() => copyText('local', parsedDate!.toString())}
            class="hover:text-slate-200 p-0.5"
          >
            {#if copiedKey === 'local'}
              <Check size={12} class="text-emerald-500" />
            {:else}
              <Copy size={12} />
            {/if}
          </button>
        </div>
        <div class="font-mono text-xs text-emerald-600 dark:text-emerald-400 select-all">
          {formatForTimezone(parsedDate, Intl.DateTimeFormat().resolvedOptions().timeZone)}
        </div>
        <div class="text-[10px] text-slate-500">
          Local Offset: {parsedDate.getTimezoneOffset() > 0 ? `-${parsedDate.getTimezoneOffset() / 60}h` : `+${Math.abs(parsedDate.getTimezoneOffset() / 60)}h`}
        </div>
      </div>

      <!-- 3. Unix Epoch (Seconds) -->
      <div class="bg-surface-900 border border-slate-200 dark:border-slate-800 rounded-xl p-3 flex flex-col justify-between gap-1.5">
        <div class="flex items-center justify-between text-slate-500 dark:text-slate-400 text-[11px]">
          <span class="font-bold">Unix Epoch (Seconds)</span>
          <button
            type="button"
            onclick={() => copyText('epoch_s', String(Math.floor(parsedDate!.getTime() / 1000)))}
            class="hover:text-slate-200 p-0.5"
          >
            {#if copiedKey === 'epoch_s'}
              <Check size={12} class="text-emerald-500" />
            {:else}
              <Copy size={12} />
            {/if}
          </button>
        </div>
        <div class="font-mono text-xs text-amber-500 select-all font-bold">
          {Math.floor(parsedDate.getTime() / 1000)}
        </div>
        <button
          type="button"
          onclick={() => onChange(Math.floor(parsedDate!.getTime() / 1000))}
          class="text-[10px] text-slate-500 hover:text-indigo-400 text-left cursor-pointer"
        >
          Use as numeric timestamp
        </button>
      </div>

      <!-- 4. Unix Epoch (Milliseconds) -->
      <div class="bg-surface-900 border border-slate-200 dark:border-slate-800 rounded-xl p-3 flex flex-col justify-between gap-1.5">
        <div class="flex items-center justify-between text-slate-500 dark:text-slate-400 text-[11px]">
          <span class="font-bold">Unix Epoch (Milliseconds)</span>
          <button
            type="button"
            onclick={() => copyText('epoch_ms', String(parsedDate!.getTime()))}
            class="hover:text-slate-200 p-0.5"
          >
            {#if copiedKey === 'epoch_ms'}
              <Check size={12} class="text-emerald-500" />
            {:else}
              <Copy size={12} />
            {/if}
          </button>
        </div>
        <div class="font-mono text-xs text-amber-400 select-all font-bold">
          {parsedDate.getTime()}
        </div>
        <button
          type="button"
          onclick={() => onChange(parsedDate!.getTime())}
          class="text-[10px] text-slate-500 hover:text-indigo-400 text-left cursor-pointer"
        >
          Use as ms timestamp
        </button>
      </div>
    </div>

    <!-- World Timezones List -->
    <div class="bg-surface-900 border border-slate-200 dark:border-slate-800 rounded-xl p-3 space-y-2">
      <div class="flex items-center justify-between text-slate-500 dark:text-slate-400 text-[11px] font-bold">
        <div class="flex items-center gap-1.5">
          <Globe size={13} class="text-indigo-400" />
          <span>Global Timezone Translations</span>
        </div>
      </div>

      <div class="grid grid-cols-1 sm:grid-cols-2 gap-2 text-xs font-mono">
        {#each TIMEZONES as item (item.tz)}
          <div class="flex items-center justify-between p-2 rounded-lg bg-surface-950 border border-slate-200/60 dark:border-slate-800/80">
            <div class="truncate mr-2">
              <span class="text-[10px] text-slate-500 block truncate">{item.label}</span>
              <span class="text-slate-800 dark:text-slate-200 text-[11px] font-semibold">{formatForTimezone(parsedDate, item.tz)}</span>
            </div>
            <button
              type="button"
              onclick={() => copyText(item.tz, formatForTimezone(parsedDate!, item.tz))}
              class="text-slate-400 hover:text-slate-200 p-1 rounded hover:bg-surface-800 shrink-0"
              title="Copy"
            >
              {#if copiedKey === item.tz}
                <Check size={11} class="text-emerald-500" />
              {:else}
                <Copy size={11} />
              {/if}
            </button>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>
