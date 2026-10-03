<script lang="ts">
  import { onMount } from 'svelte';
  import {
    Sparkles,
    X,
    Check,
    AlertCircle,
    CheckCircle2,
    RefreshCw,
    Server,
    Cpu,
    Key,
    ShieldCheck,
    Sliders,
    Zap,
    ExternalLink
  } from 'lucide-svelte';
  import { aiStore } from '$lib/state/ai.svelte';
  import type { AiProvider } from '$lib/api/types';
  import CustomSelect from '$lib/components/ui/CustomSelect.svelte';

  let { isOpen, onClose }: { isOpen: boolean; onClose: () => void } = $props();

  const providerOptions: { id: AiProvider; name: string; tag: string; icon: string; free: boolean }[] = [
    { id: 'ollama', name: 'Ollama (Local)', tag: 'Private • Free • Offline', icon: '🦙', free: true },
    { id: 'openai', name: 'OpenAI', tag: 'GPT-4o / GPT-4o-mini', icon: '⚡', free: false },
    { id: 'gemini', name: 'Google Gemini', tag: 'Gemini 1.5 Flash / Pro', icon: '✨', free: false },
    { id: 'deepseek', name: 'DeepSeek', tag: 'DeepSeek Coder / V3', icon: '🐳', free: false },
    { id: 'anthropic', name: 'Anthropic Claude', tag: 'Claude 3.5 Sonnet', icon: '🧠', free: false },
    { id: 'custom', name: 'Custom OpenAI-Compatible', tag: 'LocalAI / vLLM / Groq', icon: '⚙️', free: true },
  ];

  const modelPresets: Record<AiProvider, { value: string; label: string; desc?: string }[]> = {
    ollama: [],
    openai: [
      { value: 'gpt-4o-mini', label: 'gpt-4o-mini (Fast & Cost-Efficient)' },
      { value: 'gpt-4o', label: 'gpt-4o (State of the Art Reasoning)' },
      { value: 'gpt-4-turbo', label: 'gpt-4-turbo' },
    ],
    gemini: [
      { value: 'gemini-1.5-flash', label: 'gemini-1.5-flash (Ultra Fast & Free Tier)' },
      { value: 'gemini-1.5-pro', label: 'gemini-1.5-pro (High Context Window)' },
    ],
    anthropic: [
      { value: 'claude-3-5-sonnet-20241022', label: 'Claude 3.5 Sonnet (Best SQL Accuracy)' },
      { value: 'claude-3-5-haiku-20241022', label: 'Claude 3.5 Haiku (Ultra Fast)' },
    ],
    deepseek: [
      { value: 'deepseek-coder', label: 'deepseek-coder (Code Specialized)' },
      { value: 'deepseek-chat', label: 'deepseek-chat (General & Reasoning)' },
    ],
    custom: [
      { value: 'custom-model', label: 'Custom Model' },
    ]
  };

  onMount(() => {
    if (aiStore.provider === 'ollama') {
      aiStore.loadOllamaModels();
    }
  });

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape' && isOpen) {
      onClose();
    }
  }

  async function handleTest() {
    await aiStore.testConnection();
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if isOpen}
  <div 
    class="fixed inset-0 bg-slate-950/60 dark:bg-black/80 backdrop-blur-xs z-50 flex items-center justify-center p-3 sm:p-6 animate-in fade-in duration-150"
    role="dialog"
    aria-modal="true"
  >
    <div 
      class="bg-white dark:bg-surface-900 border border-slate-200 dark:border-slate-800 w-full max-w-2xl rounded-2xl shadow-2xl flex flex-col overflow-hidden text-slate-900 dark:text-slate-100 animate-in zoom-in-95 duration-200"
    >
      <!-- Modal Header -->
      <div class="px-5 py-4 border-b border-slate-200 dark:border-slate-800 flex items-center justify-between bg-slate-50/90 dark:bg-surface-950/80">
        <div class="flex items-center gap-3">
          <div class="p-2 bg-gradient-to-tr from-violet-600 to-indigo-600 text-white rounded-xl shadow-md shadow-violet-500/20 shrink-0">
            <Sparkles size={18} />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h3 class="font-bold text-sm text-slate-900 dark:text-white">
                AI Copilot & NL-to-SQL Settings
              </h3>
              <span class="text-[10px] font-mono px-2 py-0.5 rounded-md bg-emerald-500/10 text-emerald-700 dark:text-emerald-400 font-bold border border-emerald-500/30 flex items-center gap-1">
                <ShieldCheck size={11} /> Zero Data Leak
              </span>
            </div>
            <p class="text-[11px] text-slate-600 dark:text-slate-400">
              Configure local Ollama instance or Cloud AI API keys for natural language SQL generation
            </p>
          </div>
        </div>
        <button 
          type="button"
          onclick={onClose} 
          class="p-1.5 text-slate-500 hover:text-slate-800 dark:text-slate-400 dark:hover:text-white rounded-lg hover:bg-slate-200/80 dark:hover:bg-surface-800 transition-colors cursor-pointer" 
          title="Close (Esc)"
        >
          <X size={16} />
        </button>
      </div>

      <!-- Test Connection Result Alert Banner (Placed at top so it is immediately visible) -->
      {#if aiStore.testResult}
        <div class="mx-5 mt-4 p-3 rounded-xl flex items-start justify-between gap-3 text-xs {aiStore.testResult.success ? 'bg-emerald-500/10 dark:bg-emerald-500/15 border border-emerald-500/30 dark:border-emerald-500/40 text-emerald-950 dark:text-emerald-200' : 'bg-rose-500/10 dark:bg-rose-500/15 border border-rose-500/30 dark:border-rose-500/40 text-rose-950 dark:text-rose-200'} animate-in fade-in duration-150">
          <div class="flex items-start gap-2.5 min-w-0">
            {#if aiStore.testResult.success}
              <CheckCircle2 size={17} class="shrink-0 text-emerald-600 dark:text-emerald-400 mt-0.5" />
              <div class="space-y-0.5 min-w-0">
                <span class="font-bold text-emerald-900 dark:text-emerald-300 block">AI Connection Successful!</span>
                <p class="text-[11px] text-emerald-800 dark:text-emerald-300/90 leading-snug break-words">{aiStore.testResult.message}</p>
              </div>
            {:else}
              <AlertCircle size={17} class="shrink-0 text-rose-600 dark:text-rose-400 mt-0.5" />
              <div class="space-y-0.5 min-w-0">
                <span class="font-bold text-rose-900 dark:text-rose-300 block">AI Connection Failed</span>
                <p class="text-[11px] text-rose-800 dark:text-rose-200/90 leading-snug break-words whitespace-pre-wrap">{aiStore.testResult.message}</p>
              </div>
            {/if}
          </div>
          <button 
            type="button" 
            onclick={() => aiStore.testResult = null}
            class="text-slate-400 hover:text-slate-700 dark:hover:text-slate-100 hover:bg-slate-200 dark:hover:bg-surface-800 p-1 rounded-md transition-colors shrink-0 cursor-pointer"
            title="Dismiss"
          >
            <X size={14} />
          </button>
        </div>
      {/if}

      <!-- Modal Body -->
      <div class="p-5 space-y-5 overflow-y-auto max-h-[75vh] text-xs bg-slate-50/50 dark:bg-surface-950/40">
        <!-- 1. Provider Cards Grid -->
        <div>
          <span class="font-bold text-slate-800 dark:text-slate-200 text-xs block mb-2">
            1. Select AI Provider
          </span>
          <div class="grid grid-cols-2 sm:grid-cols-3 gap-2.5">
            {#each providerOptions as p}
              {@const isSelected = aiStore.provider === p.id}
              <button
                type="button"
                onclick={() => aiStore.provider = p.id}
                class="p-3 rounded-xl border text-left transition-all cursor-pointer flex flex-col justify-between {isSelected ? 'bg-violet-50/90 dark:bg-violet-950/40 border-violet-500 text-slate-900 dark:text-white shadow-xs ring-1 ring-violet-500/30' : 'bg-white dark:bg-surface-900 border-slate-200 dark:border-slate-800 text-slate-700 dark:text-slate-300 hover:border-slate-300 dark:hover:border-slate-700 hover:bg-slate-50/60'}"
              >
                <div class="flex items-center justify-between mb-2">
                  <span class="text-lg">{p.icon}</span>
                  {#if isSelected}
                    <span class="w-2 h-2 rounded-full bg-violet-600 shadow-xs"></span>
                  {/if}
                </div>
                <div>
                  <div class="font-bold text-xs text-slate-900 dark:text-slate-100 flex items-center justify-between">
                    <span>{p.name}</span>
                    {#if p.free}
                      <span class="text-[9px] font-sans px-1.5 py-0.2 rounded bg-emerald-500/15 text-emerald-700 dark:text-emerald-400 font-bold">Free</span>
                    {/if}
                  </div>
                  <div class="text-[10px] text-slate-500 dark:text-slate-400 truncate mt-0.5">{p.tag}</div>
                </div>
              </button>
            {/each}
          </div>
        </div>

        <!-- 2. Provider Credentials & Endpoint Config -->
        <div class="bg-white dark:bg-surface-900 p-4 rounded-xl border border-slate-200 dark:border-slate-800 shadow-xs space-y-4">
          <span class="font-bold text-slate-800 dark:text-slate-200 text-xs block">
            2. Configuration for <span class="text-violet-600 dark:text-violet-400 uppercase font-mono">{aiStore.provider}</span>
          </span>

          {#if aiStore.provider === 'ollama'}
            <!-- Ollama Specific Settings -->
            <div class="space-y-3">
              <div>
                <label for="ollama-endpoint" class="text-[11px] font-semibold text-slate-700 dark:text-slate-300 block mb-1">Ollama Host Endpoint</label>
                <div class="flex gap-2">
                  <input
                    id="ollama-endpoint"
                    type="text"
                    bind:value={aiStore.endpoint}
                    placeholder="http://localhost:11434"
                    class="flex-1 bg-slate-50 dark:bg-surface-950 border border-slate-200 dark:border-slate-700 rounded-lg px-3 py-1.5 font-mono text-xs text-slate-900 dark:text-slate-100 focus:outline-none focus:border-violet-500"
                  />
                  <button
                    type="button"
                    onclick={() => aiStore.loadOllamaModels()}
                    disabled={aiStore.isLoadingOllama}
                    class="px-3 py-1.5 rounded-lg border border-slate-200 dark:border-slate-700 bg-slate-100 hover:bg-slate-200 dark:bg-surface-800 dark:hover:bg-surface-700 text-slate-700 dark:text-slate-200 flex items-center gap-1.5 cursor-pointer font-semibold shadow-xs"
                    title="Fetch local models from Ollama"
                  >
                    <RefreshCw size={13} class={aiStore.isLoadingOllama ? 'animate-spin text-violet-500' : ''} />
                    <span>Detect Models</span>
                  </button>
                </div>
              </div>

              <div>
                <label for="ollama-model-select" class="text-[11px] font-semibold text-slate-700 dark:text-slate-300 block mb-1">Select Model</label>
                {#if aiStore.ollamaModels.length > 0}
                  <CustomSelect
                    id="ollama-model-select"
                    bind:value={aiStore.model}
                    options={aiStore.ollamaModels.map(m => ({
                      value: m.name,
                      label: m.name,
                      badge: `${(m.size / (1024 * 1024 * 1024)).toFixed(1)} GB`
                    }))}
                  />
                {:else}
                  <div class="flex items-center gap-2">
                    <input
                      type="text"
                      bind:value={aiStore.model}
                      placeholder="e.g. qwen2.5-coder:latest, llama3.2"
                      class="flex-1 bg-slate-50 dark:bg-surface-950 border border-slate-200 dark:border-slate-700 rounded-lg px-3 py-1.5 font-mono text-xs text-slate-900 dark:text-slate-100 focus:outline-none focus:border-violet-500"
                    />
                  </div>
                  <p class="text-[10px] text-slate-500 mt-1">
                    No active models detected yet. Make sure Ollama is running (`ollama run qwen2.5-coder`) or click Detect Models.
                  </p>
                {/if}
              </div>
            </div>
          {:else}
            <!-- Cloud & Custom Settings -->
            <div class="space-y-3">
              {#if aiStore.provider !== 'custom'}
                <div>
                  <label for="ai-apikey" class="text-[11px] font-semibold text-slate-700 dark:text-slate-300 block mb-1">
                    API Key (Stored Locally & Encrypted)
                  </label>
                  <div class="relative">
                    <input
                      id="ai-apikey"
                      type="password"
                      bind:value={aiStore.apiKey}
                      placeholder="sk-..."
                      class="w-full bg-slate-50 dark:bg-surface-950 border border-slate-200 dark:border-slate-700 rounded-lg px-3 py-1.5 font-mono text-xs text-slate-900 dark:text-slate-100 focus:outline-none focus:border-violet-500"
                    />
                  </div>
                </div>
              {/if}

              <div>
                <label for="cloud-model-select" class="text-[11px] font-semibold text-slate-700 dark:text-slate-300 block mb-1">Model Name</label>
                {#if modelPresets[aiStore.provider]?.length > 0}
                  <CustomSelect
                    id="cloud-model-select"
                    bind:value={aiStore.model}
                    options={modelPresets[aiStore.provider]}
                  />
                {:else}
                  <input
                    type="text"
                    bind:value={aiStore.model}
                    placeholder="Model identifier (e.g. custom-model)"
                    class="w-full bg-slate-50 dark:bg-surface-950 border border-slate-200 dark:border-slate-700 rounded-lg px-3 py-1.5 font-mono text-xs text-slate-900 dark:text-slate-100 focus:outline-none focus:border-violet-500"
                  />
                {/if}
              </div>

              {#if aiStore.provider === 'custom' || aiStore.provider === 'openai'}
                <div>
                  <label for="custom-endpoint" class="text-[11px] font-semibold text-slate-700 dark:text-slate-300 block mb-1">Custom Base Endpoint (Optional)</label>
                  <input
                    id="custom-endpoint"
                    type="text"
                    bind:value={aiStore.endpoint}
                    placeholder="https://api.openai.com/v1"
                    class="w-full bg-slate-50 dark:bg-surface-950 border border-slate-200 dark:border-slate-700 rounded-lg px-3 py-1.5 font-mono text-xs text-slate-900 dark:text-slate-100 focus:outline-none focus:border-violet-500"
                  />
                </div>
              {/if}
            </div>
          {/if}

          <!-- Temperature Control -->
          <div class="pt-3 border-t border-slate-200 dark:border-slate-800/80 flex items-center justify-between">
            <div>
              <span class="text-[11px] font-semibold text-slate-700 dark:text-slate-300 block">Temperature: {aiStore.temperature}</span>
              <span class="text-[10px] text-slate-500">Lower values (0.1 - 0.3) generate more deterministic & accurate SQL syntax</span>
            </div>
            <input
              type="range"
              min="0.0"
              max="1.0"
              step="0.05"
              bind:value={aiStore.temperature}
              class="w-32 accent-violet-600 cursor-pointer"
            />
          </div>
        </div>

        <!-- Privacy & Security Notice Banner -->
        <div class="p-3 bg-emerald-500/10 border border-emerald-500/30 rounded-xl text-emerald-800 dark:text-emerald-300 text-xs flex items-start gap-2.5">
          <ShieldCheck size={16} class="shrink-0 mt-0.5 text-emerald-600 dark:text-emerald-400" />
          <div class="space-y-0.5">
            <strong class="font-bold block">Privacy-First Architecture Guarantee:</strong>
            <p class="text-[11px] text-emerald-700 dark:text-emerald-400 leading-relaxed">
              FugDB <strong>never transmits your database rows or personal data</strong> to AI providers. Only the table names, column names, and data types (schema metadata) are shared to format accurate queries.
            </p>
          </div>
        </div>

      </div>

      <!-- Modal Footer -->
      <div class="px-5 py-3.5 border-t border-slate-200 dark:border-slate-800 bg-slate-50/90 dark:bg-surface-950/80 flex items-center justify-between">
        <button
          type="button"
          onclick={handleTest}
          disabled={aiStore.isTesting}
          class="px-3.5 py-1.5 rounded-lg border border-slate-200 dark:border-slate-700 bg-white hover:bg-slate-100 dark:bg-surface-800 dark:hover:bg-surface-700 text-slate-800 dark:text-slate-200 text-xs font-semibold flex items-center gap-1.5 cursor-pointer shadow-xs disabled:opacity-50 transition-colors"
        >
          {#if aiStore.isTesting}
            <RefreshCw size={13} class="animate-spin text-violet-500" />
            <span>Testing Connection...</span>
          {:else}
            <Zap size={13} class="text-violet-500" />
            <span>Test Connection</span>
          {/if}
        </button>

        <div class="flex items-center gap-2">
          <button
            type="button"
            onclick={onClose}
            class="px-4 py-1.5 rounded-lg bg-violet-600 hover:bg-violet-500 text-white text-xs font-semibold cursor-pointer shadow-md shadow-violet-600/20 transition-all flex items-center gap-1.5"
          >
            <Check size={14} />
            <span>Save & Close</span>
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
