<script lang="ts">
  import { ChevronDown, Check } from 'lucide-svelte';

  export interface SelectOption<T = any> {
    value: T;
    label: string;
    desc?: string;
    badge?: string;
    badgeColor?: string;
    icon?: any;
    dotColor?: string;
  }

  let { 
    id,
    options = [],
    value = $bindable(),
    label = '',
    placeholder = 'Select option...',
    fontMono = false,
    disabled = false,
    size = 'md',
    menuClass = '',
    onchange
  }: {
    id?: string;
    options: SelectOption[];
    value: any;
    label?: string;
    placeholder?: string;
    fontMono?: boolean;
    disabled?: boolean;
    size?: 'sm' | 'md';
    menuClass?: string;
    onchange?: (val: any) => void;
  } = $props();

  let isOpen = $state(false);
  let containerRef: HTMLDivElement | null = $state(null);
  let menuRef: HTMLDivElement | null = $state(null);
  let menuPos = $state<{ top: number; left: number; width: number }>({ top: 0, left: 0, width: 0 });

  const selectedOption = $derived(
    options.find(o => String(o.value) === String(value))
  );

  function updateMenuPosition() {
    if (!containerRef) return;
    const rect = containerRef.getBoundingClientRect();
    menuPos = {
      top: rect.bottom + 4,
      left: rect.left,
      width: rect.width
    };
  }

  function toggleDropdown() {
    if (disabled) return;
    if (!isOpen) {
      updateMenuPosition();
    }
    isOpen = !isOpen;
  }

  function handleSelect(optValue: any) {
    value = optValue;
    isOpen = false;
    onchange?.(optValue);
  }

  function handleWindowPointerDown(e: PointerEvent) {
    if (isOpen) {
      const target = e.target as Node;
      if (containerRef && containerRef.contains(target)) return;
      if (menuRef && menuRef.contains(target)) return;
      isOpen = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && isOpen) {
      isOpen = false;
      e.stopPropagation();
    }
  }

  $effect(() => {
    if (isOpen) {
      updateMenuPosition();
      const handleScroll = () => updateMenuPosition();
      window.addEventListener('scroll', handleScroll, true);
      window.addEventListener('resize', handleScroll);
      return () => {
        window.removeEventListener('scroll', handleScroll, true);
        window.removeEventListener('resize', handleScroll);
      };
    }
  });
</script>

<svelte:window onpointerdown={handleWindowPointerDown} onkeydown={handleKeydown} />

<div bind:this={containerRef} class="relative w-full select-none text-left">
  {#if label}
    <label for={id} class="font-semibold text-slate-700 dark:text-slate-300 text-[11px] block mb-1">{label}</label>
  {/if}

  <!-- Trigger Button -->
  <button
    {id}
    type="button"
    {disabled}
    onclick={toggleDropdown}
    class="w-full bg-slate-50 hover:bg-slate-100/80 dark:bg-surface-900 dark:hover:bg-surface-800/80 border border-slate-200 dark:border-slate-700 hover:border-slate-300 dark:hover:border-slate-600 rounded-lg {size === 'sm' ? 'px-2.5 py-1 text-[11px]' : 'px-3 py-1.5 text-xs'} flex items-center justify-between text-slate-900 dark:text-slate-100 transition-all focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 shadow-xs cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed {fontMono ? 'font-mono' : ''}"
  >
    <div class="flex items-center gap-2 truncate pr-2">
      {#if selectedOption?.dotColor}
        <span class="w-2 h-2 rounded-full {selectedOption.dotColor} shadow-xs shrink-0"></span>
      {/if}
      <span class="truncate font-medium {selectedOption ? 'text-slate-900 dark:text-slate-100' : 'text-slate-400 dark:text-slate-500'}">
        {selectedOption ? selectedOption.label : placeholder}
      </span>
      {#if selectedOption?.badge}
        <span class="text-[10px] font-sans px-1.5 py-0.2 rounded font-semibold {selectedOption.badgeColor || 'bg-slate-200 dark:bg-slate-800 text-slate-700 dark:text-slate-400'}">
          {selectedOption.badge}
        </span>
      {/if}
    </div>

    <ChevronDown size={14} class="text-slate-400 dark:text-slate-500 transition-transform duration-200 shrink-0 {isOpen ? 'rotate-180 text-indigo-600 dark:text-indigo-400' : ''}" />
  </button>

  <!-- Options Popover: Fixed positioning with z-[9999] so it floats freely OVER modal footers and ignores overflow clipping -->
  {#if isOpen}
    <div 
      bind:this={menuRef}
      style="position: fixed; top: {menuPos.top}px; left: {menuPos.left}px; width: {menuPos.width}px; z-index: 9999;"
      class="bg-white dark:bg-surface-900 border border-slate-200 dark:border-slate-700 rounded-xl shadow-2xl p-1 space-y-0.5 max-h-60 overflow-y-auto animate-in fade-in zoom-in-95 duration-100 {menuClass}"
    >
      {#each options as opt}
        {@const isSelected = String(opt.value) === String(value)}
        <button
          type="button"
          onclick={() => handleSelect(opt.value)}
          class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-xs transition-colors cursor-pointer {isSelected ? 'bg-indigo-50 dark:bg-indigo-600/20 text-indigo-700 dark:text-indigo-300 font-bold' : 'hover:bg-slate-100 dark:hover:bg-surface-800 text-slate-700 dark:text-slate-300'} {fontMono ? 'font-mono' : ''}"
        >
          <div class="flex items-center gap-2 truncate">
            {#if opt.dotColor}
              <span class="w-2 h-2 rounded-full {opt.dotColor} shadow-xs shrink-0"></span>
            {/if}
            <div class="truncate text-left">
              <span class="truncate block">{opt.label}</span>
              {#if opt.desc}
                <span class="text-[10px] text-slate-500 dark:text-slate-400 block font-normal">{opt.desc}</span>
              {/if}
            </div>
            {#if opt.badge}
              <span class="text-[10px] font-sans px-1.5 py-0.2 rounded font-semibold {opt.badgeColor || 'bg-slate-200 dark:bg-slate-800 text-slate-700 dark:text-slate-400'}">
                {opt.badge}
              </span>
            {/if}
          </div>

          {#if isSelected}
            <Check size={13} class="text-indigo-600 dark:text-indigo-400 shrink-0 ml-2" />
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</div>
