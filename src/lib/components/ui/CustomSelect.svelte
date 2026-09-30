<script lang="ts">
  import { ChevronDown, Check } from 'lucide-svelte';

  interface SelectOption {
    value: string;
    label: string;
    badge?: string;
    badgeColor?: string;
    icon?: any;
    dotColor?: string;
  }

  let { 
    id,
    options = [],
    value = $bindable(''),
    label = '',
    placeholder = 'Select option...',
    onchange
  }: {
    id?: string;
    options: SelectOption[];
    value: string;
    label?: string;
    placeholder?: string;
    onchange?: (val: string) => void;
  } = $props();

  let isOpen = $state(false);

  const selectedOption = $derived(
    options.find(o => o.value === value)
  );

  function handleSelect(optValue: string) {
    value = optValue;
    isOpen = false;
    onchange?.(optValue);
  }
</script>

<div class="relative w-full select-none">
  {#if label}
    <label for={id} class="font-semibold text-slate-300 text-[11px] block mb-1">{label}</label>
  {/if}

  <!-- Trigger Button -->
  <button
    {id}
    type="button"
    onclick={() => isOpen = !isOpen}
    class="w-full bg-surface-950/90 border border-slate-700/80 hover:border-indigo-500/80 rounded-lg px-3 py-1.5 flex items-center justify-between text-xs text-slate-200 transition-all focus:outline-none focus:ring-1 focus:ring-indigo-500 shadow-sm"
  >
    <div class="flex items-center gap-2 truncate">
      {#if selectedOption?.dotColor}
        <span class="w-2 h-2 rounded-full {selectedOption.dotColor} shadow-sm shrink-0"></span>
      {/if}
      <span class="truncate font-medium {selectedOption ? 'text-slate-100' : 'text-slate-500'}">
        {selectedOption ? selectedOption.label : placeholder}
      </span>
      {#if selectedOption?.badge}
        <span class="text-[10px] px-1.5 py-0.2 rounded font-semibold {selectedOption.badgeColor || 'bg-slate-800 text-slate-400'}">
          {selectedOption.badge}
        </span>
      {/if}
    </div>

    <ChevronDown size={14} class="text-slate-400 transition-transform duration-200 shrink-0 {isOpen ? 'rotate-180 text-indigo-400' : ''}" />
  </button>

  <!-- Options Popover -->
  {#if isOpen}
    <div 
      class="fixed inset-0 z-40" 
      onclick={() => isOpen = false}
      role="presentation"
    ></div>

    <div class="absolute left-0 right-0 top-full mt-1.5 z-50 bg-surface-900 border border-slate-700 rounded-xl shadow-2xl p-1 space-y-0.5 max-h-60 overflow-y-auto">
      {#each options as opt}
        <button
          type="button"
          onclick={() => handleSelect(opt.value)}
          class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-xs transition-colors {value === opt.value ? 'bg-indigo-600/20 text-indigo-300 font-semibold border border-indigo-500/30' : 'hover:bg-surface-800 text-slate-300'}"
        >
          <div class="flex items-center gap-2 truncate">
            {#if opt.dotColor}
              <span class="w-2 h-2 rounded-full {opt.dotColor} shadow-sm shrink-0"></span>
            {/if}
            <span class="truncate">{opt.label}</span>
          </div>

          {#if value === opt.value}
            <Check size={13} class="text-indigo-400 shrink-0 ml-2" />
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</div>
