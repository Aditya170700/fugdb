<script lang="ts">
  import { 
    Binary, 
    Hash, 
    Copy, 
    Check, 
    RefreshCw, 
    ShieldCheck, 
    Sparkles,
    FileCode,
    Cpu
  } from 'lucide-svelte';

  let { 
    value, 
    onChange 
  }: { 
    value: any; 
    onChange: (newVal: any) => void; 
  } = $props();

  let mode = $state<'uuid' | 'base64' | 'hex'>('uuid');
  let rawStr = $state<string>('');
  let copiedKey = $state<string | null>(null);

  $effect(() => {
    rawStr = String(value ?? '');
    // Auto-select mode based on content
    if (isUuid(rawStr)) {
      mode = 'uuid';
    } else if (isHex(rawStr)) {
      mode = 'hex';
    } else {
      mode = 'base64';
    }
  });

  function isUuid(str: string): boolean {
    return /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(str.trim());
  }

  function getUuidVersion(str: string): string {
    const clean = str.trim();
    if (!isUuid(clean)) return 'Invalid UUID';
    const ver = clean.charAt(14);
    if (ver === '1') return 'UUID Version 1 (Time-based & MAC)';
    if (ver === '2') return 'UUID Version 2 (DCE Security)';
    if (ver === '3') return 'UUID Version 3 (MD5 Name-based)';
    if (ver === '4') return 'UUID Version 4 (Cryptographically Random)';
    if (ver === '5') return 'UUID Version 5 (SHA-1 Name-based)';
    if (ver === '7') return 'UUID Version 7 (Unix Epoch Time-ordered)';
    return `UUID Version ${ver}`;
  }

  function generateUuidV4(): string {
    if (typeof crypto !== 'undefined' && crypto.randomUUID) {
      return crypto.randomUUID();
    }
    return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
      const r = Math.random() * 16 | 0;
      const v = c === 'x' ? r : (r & 0x3 | 0x8);
      return v.toString(16);
    });
  }

  function isHex(str: string): boolean {
    const s = str.trim().replace(/^0x/, '');
    return s.length > 0 && s.length % 2 === 0 && /^[0-9a-fA-F]+$/.test(s);
  }

  function stringToBase64(str: string): string {
    try {
      return btoa(unescape(encodeURIComponent(str)));
    } catch {
      return '';
    }
  }

  function base64ToString(b64: string): string {
    try {
      return decodeURIComponent(escape(atob(b64)));
    } catch {
      return 'Unable to decode Base64 to valid UTF-8 string';
    }
  }

  function stringToHex(str: string): string {
    let hex = '';
    for (let i = 0; i < str.length; i++) {
      hex += str.charCodeAt(i).toString(16).padStart(2, '0');
    }
    return hex;
  }

  function hexToString(hex: string): string {
    try {
      const clean = hex.trim().replace(/^0x/, '').replace(/\s+/g, '');
      let str = '';
      for (let i = 0; i < clean.length; i += 2) {
        str += String.fromCharCode(parseInt(clean.substr(i, 2), 16));
      }
      return str;
    } catch {
      return '';
    }
  }

  function generateHexDump(str: string): Array<{ offset: string; hex: string; ascii: string }> {
    const bytes = new TextEncoder().encode(str);
    const dump = [];
    const chunkSize = 16;

    for (let i = 0; i < bytes.length; i += chunkSize) {
      const slice = bytes.slice(i, i + chunkSize);
      const offset = i.toString(16).padStart(8, '0');

      let hexPart = '';
      let asciiPart = '';

      for (let j = 0; j < chunkSize; j++) {
        if (j < slice.length) {
          const b = slice[j];
          hexPart += b.toString(16).padStart(2, '0') + ' ';
          asciiPart += (b >= 32 && b <= 126) ? String.fromCharCode(b) : '.';
        } else {
          hexPart += '   ';
        }
      }

      dump.push({ offset, hex: hexPart.trim(), ascii: asciiPart });
    }

    return dump;
  }

  function copyText(key: string, text: string) {
    navigator.clipboard.writeText(text);
    copiedKey = key;
    setTimeout(() => copiedKey = null, 1500);
  }
</script>

<!-- Subtab Selector -->
<div class="p-3 border-b border-slate-200 dark:border-slate-800 bg-surface-900/60 flex items-center justify-between text-xs">
  <div class="flex items-center gap-1 bg-surface-950 p-0.5 rounded-lg border border-slate-200 dark:border-slate-800">
    <button
      type="button"
      onclick={() => mode = 'uuid'}
      class="flex items-center gap-1.5 px-2.5 py-1 rounded-md font-semibold text-[11px] transition-colors cursor-pointer {mode === 'uuid' ? 'bg-indigo-600 text-white shadow-xs' : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white'}"
    >
      <ShieldCheck size={12} />
      <span>UUID Inspector</span>
    </button>

    <button
      type="button"
      onclick={() => mode = 'base64'}
      class="flex items-center gap-1.5 px-2.5 py-1 rounded-md font-semibold text-[11px] transition-colors cursor-pointer {mode === 'base64' ? 'bg-indigo-600 text-white shadow-xs' : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white'}"
    >
      <Binary size={12} />
      <span>Base64</span>
    </button>

    <button
      type="button"
      onclick={() => mode = 'hex'}
      class="flex items-center gap-1.5 px-2.5 py-1 rounded-md font-semibold text-[11px] transition-colors cursor-pointer {mode === 'hex' ? 'bg-indigo-600 text-white shadow-xs' : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white'}"
    >
      <Cpu size={12} />
      <span>Hex Dump</span>
    </button>
  </div>
</div>

<div class="flex-1 overflow-auto p-4 space-y-4 bg-surface-950 text-xs font-mono select-text">
  {#if mode === 'uuid'}
    {@const valid = isUuid(rawStr)}
    <div class="p-4 bg-surface-900 border border-slate-200 dark:border-slate-800 rounded-xl space-y-3">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <ShieldCheck size={16} class={valid ? 'text-emerald-500' : 'text-amber-500'} />
          <span class="font-bold text-slate-800 dark:text-slate-100 font-sans">
            {valid ? 'Valid RFC 4122 UUID' : 'Not a Standard UUID'}
          </span>
        </div>
        <span class="text-[11px] text-slate-500">{getUuidVersion(rawStr)}</span>
      </div>

      <div class="bg-surface-950 p-3 rounded-lg border border-slate-200 dark:border-slate-800 flex items-center justify-between gap-2">
        <span class="text-indigo-600 dark:text-indigo-300 font-bold select-all text-xs truncate">
          {rawStr || '(empty)'}
        </span>
        <button
          type="button"
          onclick={() => copyText('uuid', rawStr)}
          class="p-1 text-slate-400 hover:text-white"
        >
          {#if copiedKey === 'uuid'}
            <Check size={12} class="text-emerald-500" />
          {:else}
            <Copy size={12} />
          {/if}
        </button>
      </div>

      <div class="flex items-center gap-2 pt-2">
        <button
          type="button"
          onclick={() => {
            const newId = generateUuidV4();
            rawStr = newId;
            onChange(newId);
          }}
          class="flex items-center gap-1.5 px-3 py-1.5 bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg text-xs font-semibold cursor-pointer font-sans"
        >
          <RefreshCw size={12} />
          <span>Generate New Random UUID (v4)</span>
        </button>
      </div>
    </div>
  {:else if mode === 'base64'}
    <div class="space-y-3">
      <!-- Decoded Text -->
      <div class="bg-surface-900 border border-slate-200 dark:border-slate-800 rounded-xl p-3 space-y-1.5">
        <div class="flex items-center justify-between text-slate-500 text-[11px] font-sans">
          <span class="font-bold">Decoded UTF-8 Text</span>
          <button
            type="button"
            onclick={() => copyText('b64_dec', base64ToString(rawStr))}
            class="hover:text-slate-200"
          >
            {#if copiedKey === 'b64_dec'}
              <Check size={12} class="text-emerald-500" />
            {:else}
              <Copy size={12} />
            {/if}
          </button>
        </div>
        <div class="bg-surface-950 p-2.5 rounded-lg border border-slate-200 dark:border-slate-800/80 text-emerald-600 dark:text-emerald-400 whitespace-pre-wrap break-all max-h-40 overflow-auto">
          {base64ToString(rawStr)}
        </div>
      </div>

      <!-- Base64 Encoded Text -->
      <div class="bg-surface-900 border border-slate-200 dark:border-slate-800 rounded-xl p-3 space-y-1.5">
        <div class="flex items-center justify-between text-slate-500 text-[11px] font-sans">
          <span class="font-bold">Base64 Encoded</span>
          <button
            type="button"
            onclick={() => copyText('b64_enc', stringToBase64(rawStr))}
            class="hover:text-slate-200"
          >
            {#if copiedKey === 'b64_enc'}
              <Check size={12} class="text-emerald-500" />
            {:else}
              <Copy size={12} />
            {/if}
          </button>
        </div>
        <div class="bg-surface-950 p-2.5 rounded-lg border border-slate-200 dark:border-slate-800/80 text-indigo-600 dark:text-indigo-300 whitespace-pre-wrap break-all max-h-40 overflow-auto font-mono">
          {stringToBase64(rawStr)}
        </div>
      </div>
    </div>
  {:else}
    <!-- Hex Dump View -->
    <div class="bg-surface-900 border border-slate-200 dark:border-slate-800 rounded-xl p-3 space-y-2">
      <div class="flex items-center justify-between text-slate-500 text-[11px] font-sans pb-1 border-b border-slate-200 dark:border-slate-800">
        <span class="font-bold">Hexadecimal Memory Dump</span>
        <span>{new TextEncoder().encode(rawStr).length} Bytes</span>
      </div>

      <div class="bg-surface-950 p-2.5 rounded-lg border border-slate-200 dark:border-slate-800/80 font-mono text-[11px] overflow-auto max-h-80 space-y-1">
        <div class="text-slate-500 flex gap-4 border-b border-slate-800 pb-1 font-semibold select-none">
          <span class="w-16">Offset</span>
          <span class="flex-1">00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F</span>
          <span class="w-32">ASCII</span>
        </div>
        {#each generateHexDump(rawStr) as line}
          <div class="flex gap-4 hover:bg-surface-900/60 py-0.5 rounded">
            <span class="w-16 text-slate-500 select-none">{line.offset}</span>
            <span class="flex-1 text-indigo-400 tracking-wider">{line.hex}</span>
            <span class="w-32 text-emerald-400 select-all">{line.ascii}</span>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>
