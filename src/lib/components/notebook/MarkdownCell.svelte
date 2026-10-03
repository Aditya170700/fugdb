<script lang="ts">
  import type { MarkdownNotebookCell } from '$lib/api/types';
  import { notebookStore } from '$lib/state/notebook.svelte';
  import { renderMarkdown } from '$lib/utils/notebookExport';
  import { 
    Edit2, 
    Check, 
    Trash2, 
    ArrowUp, 
    ArrowDown, 
    Copy, 
    Bold, 
    Italic, 
    Code, 
    List, 
    Heading, 
    Quote, 
    Table,
    FileText
  } from 'lucide-svelte';

  let { 
    notebookId, 
    cell, 
    index, 
    totalCells 
  }: { 
    notebookId: string; 
    cell: MarkdownNotebookCell; 
    index: number; 
    totalCells: number; 
  } = $props();

  let isEditing = $state(false);
  let localContent = $state('');
  let textareaRef = $state<HTMLTextAreaElement | null>(null);

  $effect(() => {
    isEditing = cell.isEditing ?? false;
    localContent = cell.content;
  });

  function saveAndExit() {
    notebookStore.updateMarkdownContent(notebookId, cell.id, localContent);
    isEditing = false;
  }

  function startEditing() {
    isEditing = true;
    setTimeout(() => {
      textareaRef?.focus();
    }, 50);
  }

  function insertFormatting(prefix: string, suffix: string = '', placeholder: string = '') {
    if (!textareaRef) return;
    const start = textareaRef.selectionStart;
    const end = textareaRef.selectionEnd;
    const selected = localContent.substring(start, end) || placeholder;
    const replacement = prefix + selected + suffix;
    localContent = localContent.substring(0, start) + replacement + localContent.substring(end);
    notebookStore.updateMarkdownContent(notebookId, cell.id, localContent);

    setTimeout(() => {
      if (textareaRef) {
        textareaRef.focus();
        textareaRef.setSelectionRange(start + prefix.length, start + prefix.length + selected.length);
      }
    }, 30);
  }

  function handleKeyDown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
      e.preventDefault();
      saveAndExit();
    } else if (e.key === 'Escape') {
      isEditing = false;
    }
  }
</script>

<div class="group relative rounded-xl border border-slate-200/80 dark:border-slate-800/80 bg-white dark:bg-surface-900/90 shadow-xs hover:border-emerald-500/50 dark:hover:border-emerald-500/40 transition-all overflow-hidden mb-4">
  
  <!-- Cell Header / Control Bar -->
  <div class="px-4 py-2 bg-slate-50/80 dark:bg-surface-950/80 border-b border-slate-200/80 dark:border-slate-800/80 flex items-center justify-between gap-2 text-xs">
    <div class="flex items-center gap-2">
      <span class="flex items-center gap-1.5 px-2 py-0.5 rounded-md bg-emerald-500/15 text-emerald-700 dark:text-emerald-300 font-bold text-[10.5px] border border-emerald-500/30">
        <FileText size={11} />
        <span>MARKDOWN</span>
      </span>
      <span class="text-[11px] text-slate-400 font-mono">#{index + 1}</span>
    </div>

    <!-- Right Actions -->
    <div class="flex items-center gap-1 opacity-70 group-hover:opacity-100 transition-opacity">
      {#if isEditing}
        <button
          type="button"
          onclick={saveAndExit}
          class="flex items-center gap-1 px-2.5 py-1 rounded-md bg-emerald-600 hover:bg-emerald-500 text-white font-semibold text-[11px] shadow-xs transition-all cursor-pointer"
          title="Done editing (Cmd+Enter)"
        >
          <Check size={12} />
          <span>Done</span>
        </button>
      {:else}
        <button
          type="button"
          onclick={startEditing}
          class="flex items-center gap-1 px-2 py-1 rounded-md bg-surface-800 hover:bg-surface-700 text-slate-700 dark:text-slate-300 border border-slate-300 dark:border-slate-700 text-[11px] font-medium transition-all cursor-pointer"
          title="Edit Markdown"
        >
          <Edit2 size={11} />
          <span>Edit</span>
        </button>
      {/if}

      <div class="w-[1px] h-3.5 bg-slate-300 dark:bg-slate-700 mx-1"></div>

      <!-- Move Up / Down -->
      <button
        type="button"
        disabled={index === 0}
        onclick={() => notebookStore.moveCell(notebookId, cell.id, 'up')}
        class="p-1 rounded text-slate-500 hover:text-slate-800 dark:hover:text-slate-200 hover:bg-surface-800 disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
        title="Move Cell Up"
      >
        <ArrowUp size={13} />
      </button>

      <button
        type="button"
        disabled={index === totalCells - 1}
        onclick={() => notebookStore.moveCell(notebookId, cell.id, 'down')}
        class="p-1 rounded text-slate-500 hover:text-slate-800 dark:hover:text-slate-200 hover:bg-surface-800 disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
        title="Move Cell Down"
      >
        <ArrowDown size={13} />
      </button>

      <!-- Duplicate -->
      <button
        type="button"
        onclick={() => notebookStore.duplicateCell(notebookId, cell.id)}
        class="p-1 rounded text-slate-500 hover:text-slate-800 dark:hover:text-slate-200 hover:bg-surface-800 transition-colors"
        title="Duplicate Cell"
      >
        <Copy size={13} />
      </button>

      <!-- Delete -->
      <button
        type="button"
        onclick={() => notebookStore.deleteCell(notebookId, cell.id)}
        class="p-1 rounded text-slate-500 hover:text-rose-600 dark:hover:text-rose-400 hover:bg-rose-500/15 transition-colors"
        title="Delete Cell"
      >
        <Trash2 size={13} />
      </button>
    </div>
  </div>

  <!-- Cell Body: Rendered or Textarea -->
  <div class="p-4">
    {#if isEditing}
      <!-- Formatting Toolbar -->
      <div class="flex items-center gap-1 mb-2 pb-2 border-b border-slate-200 dark:border-slate-800 text-slate-600 dark:text-slate-400 overflow-x-auto">
        <button
          type="button"
          onclick={() => insertFormatting('# ', '', 'Heading 1')}
          class="p-1 hover:bg-surface-800 rounded text-[11px] font-bold"
          title="Heading 1"
        >
          H1
        </button>
        <button
          type="button"
          onclick={() => insertFormatting('## ', '', 'Heading 2')}
          class="p-1 hover:bg-surface-800 rounded text-[11px] font-bold"
          title="Heading 2"
        >
          H2
        </button>
        <button
          type="button"
          onclick={() => insertFormatting('### ', '', 'Heading 3')}
          class="p-1 hover:bg-surface-800 rounded text-[11px] font-bold"
          title="Heading 3"
        >
          H3
        </button>
        <div class="w-[1px] h-3 bg-slate-300 dark:bg-slate-700 mx-1"></div>
        <button
          type="button"
          onclick={() => insertFormatting('**', '**', 'bold text')}
          class="p-1 hover:bg-surface-800 rounded"
          title="Bold"
        >
          <Bold size={13} />
        </button>
        <button
          type="button"
          onclick={() => insertFormatting('*', '*', 'italic text')}
          class="p-1 hover:bg-surface-800 rounded"
          title="Italic"
        >
          <Italic size={13} />
        </button>
        <button
          type="button"
          onclick={() => insertFormatting('`', '`', 'code')}
          class="p-1 hover:bg-surface-800 rounded"
          title="Inline Code"
        >
          <Code size={13} />
        </button>
        <button
          type="button"
          onclick={() => insertFormatting('- ', '', 'List item')}
          class="p-1 hover:bg-surface-800 rounded"
          title="Unordered List"
        >
          <List size={13} />
        </button>
        <button
          type="button"
          onclick={() => insertFormatting('> ', '', 'Blockquote')}
          class="p-1 hover:bg-surface-800 rounded"
          title="Blockquote"
        >
          <Quote size={13} />
        </button>
        <button
          type="button"
          onclick={() => insertFormatting('| Column 1 | Column 2 |\n| --- | --- |\n| Value 1 | Value 2 |', '', '')}
          class="p-1 hover:bg-surface-800 rounded"
          title="Insert Table"
        >
          <Table size={13} />
        </button>
        <button
          type="button"
          onclick={() => insertFormatting('```sql\n', '\n```', 'SELECT * FROM table;')}
          class="px-1.5 py-0.5 hover:bg-surface-800 rounded text-[10.5px] font-mono font-bold text-indigo-500"
          title="SQL Code Block"
        >
          ```sql
        </button>
      </div>

      <textarea
        bind:this={textareaRef}
        bind:value={localContent}
        oninput={(e) => notebookStore.updateMarkdownContent(notebookId, cell.id, (e.target as HTMLTextAreaElement).value)}
        onkeydown={handleKeyDown}
        placeholder="Type Markdown text here... (Headings, lists, code, tables supported)"
        rows={Math.max(4, localContent.split('\n').length + 1)}
        class="w-full p-3 bg-slate-50 dark:bg-surface-950 border border-slate-300 dark:border-slate-800 rounded-lg text-slate-900 dark:text-slate-100 font-mono text-xs leading-relaxed resize-y focus:outline-none focus:ring-1 focus:ring-emerald-500"
      ></textarea>

      <div class="mt-2 flex items-center justify-between text-[11px] text-slate-400">
        <span>Press <strong>Cmd+Enter</strong> to finish editing</span>
        <button
          type="button"
          onclick={saveAndExit}
          class="text-emerald-600 dark:text-emerald-400 font-semibold hover:underline cursor-pointer"
        >
          Done Editing ✓
        </button>
      </div>
    {:else}
      <!-- Rendered Markdown View (Double click to edit) -->
      <div 
        role="button"
        tabindex="0"
        ondblclick={startEditing}
        onkeydown={(e) => { if (e.key === 'Enter') startEditing(); }}
        class="prose dark:prose-invert max-w-none select-text cursor-text text-slate-800 dark:text-slate-200 text-sm leading-relaxed"
        title="Double click to edit markdown"
      >
        {#if localContent.trim()}
          <!-- eslint-disable-next-line svelte/no-at-html-tags -->
          {@html renderMarkdown(localContent)}
        {:else}
          <div class="italic text-slate-400 py-2">
            Empty markdown cell. Click Edit or double-click to add content.
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>
