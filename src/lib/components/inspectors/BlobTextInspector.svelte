<script lang="ts">
  import { 
    FileText, 
    Eye, 
    Copy, 
    Check, 
    Download, 
    Image as ImageIcon,
    Code,
    Sparkles
  } from 'lucide-svelte';

  let { 
    value, 
    onChange 
  }: { 
    value: any; 
    onChange: (newVal: any) => void; 
  } = $props();

  let mode = $state<'text' | 'markdown' | 'image'>('text');
  let rawText = $state<string>('');
  let copied = $state<boolean>(false);

  $effect(() => {
    rawText = String(value ?? '');
    if (isImageOrSvg(rawText)) {
      mode = 'image';
    } else if (isLikelyMarkdown(rawText)) {
      mode = 'markdown';
    } else {
      mode = 'text';
    }
  });

  function isImageOrSvg(str: string): boolean {
    const s = str.trim();
    return (
      (s.startsWith('<svg') && s.endsWith('</svg>')) ||
      s.startsWith('data:image/') ||
      /\.(png|jpe?g|gif|webp|svg|bmp|ico)(\?.*)?$/i.test(s)
    );
  }

  function isLikelyMarkdown(str: string): boolean {
    return (
      str.includes('# ') ||
      str.includes('## ') ||
      str.includes('**') ||
      str.includes('- ') ||
      str.includes('```') ||
      str.includes('| --- |')
    );
  }

  // Simple safe client markdown parser
  function renderMarkdownToHtml(md: string): string {
    let html = md
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;');

    // Code blocks
    html = html.replace(/```([a-z]*)\n([\s\S]*?)```/g, '<pre class="bg-surface-950 p-3 rounded-lg border border-slate-800 my-2 overflow-x-auto font-mono text-xs"><code>$2</code></pre>');
    
    // Inline code
    html = html.replace(/`([^`]+)`/g, '<code class="bg-surface-950 px-1.5 py-0.5 rounded border border-slate-800 text-indigo-400 font-mono text-xs">$1</code>');

    // Headings
    html = html.replace(/^### (.*$)/gim, '<h3 class="text-sm font-bold text-slate-100 mt-3 mb-1">$1</h3>');
    html = html.replace(/^## (.*$)/gim, '<h2 class="text-base font-bold text-slate-100 mt-4 mb-2">$1</h2>');
    html = html.replace(/^# (.*$)/gim, '<h1 class="text-lg font-extrabold text-slate-100 mt-4 mb-2 border-b border-slate-800 pb-1">$1</h1>');

    // Bold & Italic
    html = html.replace(/\*\*([^*]+)\*\*/g, '<strong class="font-bold text-slate-100">$1</strong>');
    html = html.replace(/\*([^*]+)\*/g, '<em class="italic text-slate-300">$1</em>');

    // Unordered lists
    html = html.replace(/^\s*-\s+(.*$)/gim, '<li class="ml-4 list-disc text-slate-300">$1</li>');

    // Line breaks
    html = html.replace(/\n\n/g, '<br/><br/>');

    return html;
  }

  function copyText() {
    navigator.clipboard.writeText(rawText);
    copied = true;
    setTimeout(() => copied = false, 1500);
  }

  function downloadAsFile() {
    const ext = mode === 'markdown' ? 'md' : isImageOrSvg(rawText) && rawText.startsWith('<svg') ? 'svg' : 'txt';
    const blob = new Blob([rawText], { type: 'text/plain;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `cell_export.${ext}`;
    a.click();
    URL.revokeObjectURL(url);
  }

  const lineCount = $derived(rawText ? rawText.split('\n').length : 0);
  const wordCount = $derived(rawText ? rawText.trim().split(/\s+/).filter(Boolean).length : 0);
</script>

<!-- Top Subtab Toolbar -->
<div class="p-3 border-b border-slate-200 dark:border-slate-800 bg-surface-900/60 flex items-center justify-between text-xs">
  <div class="flex items-center gap-1 bg-surface-950 p-0.5 rounded-lg border border-slate-200 dark:border-slate-800">
    <button
      type="button"
      onclick={() => mode = 'text'}
      class="flex items-center gap-1.5 px-2.5 py-1 rounded-md font-semibold text-[11px] transition-colors cursor-pointer {mode === 'text' ? 'bg-indigo-600 text-white shadow-xs' : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white'}"
    >
      <FileText size={12} />
      <span>Plain Text</span>
    </button>

    <button
      type="button"
      onclick={() => mode = 'markdown'}
      class="flex items-center gap-1.5 px-2.5 py-1 rounded-md font-semibold text-[11px] transition-colors cursor-pointer {mode === 'markdown' ? 'bg-indigo-600 text-white shadow-xs' : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white'}"
    >
      <Eye size={12} />
      <span>Markdown Preview</span>
    </button>

    {#if isImageOrSvg(rawText)}
      <button
        type="button"
        onclick={() => mode = 'image'}
        class="flex items-center gap-1.5 px-2.5 py-1 rounded-md font-semibold text-[11px] transition-colors cursor-pointer {mode === 'image' ? 'bg-indigo-600 text-white shadow-xs' : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white'}"
      >
        <ImageIcon size={12} />
        <span>Image / SVG Preview</span>
      </button>
    {/if}
  </div>

  <div class="flex items-center gap-2">
    <span class="text-[10px] text-slate-500 font-mono">
      {rawText.length} chars • {wordCount} words • {lineCount} lines
    </span>

    <button
      type="button"
      onclick={copyText}
      class="flex items-center gap-1 px-2.5 py-1 text-slate-700 dark:text-slate-300 bg-surface-950 border border-slate-200 dark:border-slate-800 hover:bg-surface-800 rounded-md font-semibold text-[11px] shadow-xs transition-colors"
    >
      {#if copied}
        <Check size={12} class="text-emerald-500" />
        <span class="text-emerald-500">Copied</span>
      {:else}
        <Copy size={12} />
        <span>Copy</span>
      {/if}
    </button>

    <button
      type="button"
      onclick={downloadAsFile}
      class="flex items-center gap-1 px-2.5 py-1 text-slate-700 dark:text-slate-300 bg-surface-950 border border-slate-200 dark:border-slate-800 hover:bg-surface-800 rounded-md font-semibold text-[11px] shadow-xs transition-colors"
      title="Download file"
    >
      <Download size={12} />
      <span>Download</span>
    </button>
  </div>
</div>

<!-- Main Body -->
<div class="flex-1 overflow-auto p-4 bg-surface-950 select-text">
  {#if mode === 'text'}
    <textarea
      bind:value={rawText}
      oninput={() => onChange(rawText)}
      placeholder="Type or paste long text content..."
      class="w-full h-full min-h-[320px] bg-transparent text-slate-900 dark:text-slate-100 font-mono text-xs focus:outline-none resize-none leading-relaxed"
      spellcheck="false"
    ></textarea>
  {:else if mode === 'markdown'}
    <div class="prose prose-invert max-w-none text-xs leading-relaxed text-slate-800 dark:text-slate-200 space-y-2">
      <!-- eslint-disable-next-line svelte/no-at-html-tags -->
      {@html renderMarkdownToHtml(rawText)}
    </div>
  {:else if mode === 'image'}
    <div class="flex flex-col items-center justify-center p-6 bg-surface-900 border border-slate-200 dark:border-slate-800 rounded-xl space-y-3">
      {#if rawText.trim().startsWith('<svg')}
        <div class="max-w-md max-h-80 overflow-auto p-4 bg-white/5 rounded-lg border border-slate-700">
          <!-- eslint-disable-next-line svelte/no-at-html-tags -->
          {@html rawText}
        </div>
      {:else}
        <img 
          src={rawText} 
          alt="Cell media preview"
          class="max-w-md max-h-80 object-contain rounded-lg border border-slate-700 shadow-md bg-white/5 p-2"
        />
      {/if}
      <p class="text-[11px] text-slate-500 font-mono break-all max-w-lg text-center truncate">
        {rawText}
      </p>
    </div>
  {/if}
</div>
