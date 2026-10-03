<script lang="ts">
  import { onMount } from 'svelte';
  import { EditorState, Compartment } from '@codemirror/state';
  import { EditorView, keymap, lineNumbers, highlightActiveLineGutter, highlightActiveLine } from '@codemirror/view';
  import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
  import { autocompletion, completionKeymap, closeBrackets, closeBracketsKeymap } from '@codemirror/autocomplete';
  import { syntaxHighlighting, HighlightStyle } from '@codemirror/language';
  import { tags } from '@lezer/highlight';
  import { oneDark } from '@codemirror/theme-one-dark';
  import { Play, ShieldAlert, AlertTriangle, History, Sparkles, AlertCircle, X, Activity, Zap, Lock, RotateCcw, Check, Loader2 } from 'lucide-svelte';
  import { tabsStore } from '$lib/state/tabs.svelte';
  import { themeStore } from '$lib/state/theme.svelte';
  import { connectionStore } from '$lib/state/connection.svelte';
  import { sessionStore } from '$lib/state/session.svelte';
  import { historyStore } from '$lib/state/history.svelte';
  import { aiStore } from '$lib/state/ai.svelte';
  import { explainStore } from '$lib/state/explain.svelte';
  import { assessSqlRisk } from '$lib/utils/safetyGuard';
  import { createSqlLanguageSupport } from './sqlCompletion';

  let { tabId, initialSql = '' }: { tabId: string; initialSql?: string } = $props();

  let editorContainer: HTMLDivElement;
  let view: EditorView | undefined = $state();
  const themeCompartment = new Compartment();
  const languageCompartment = new Compartment();

  const currentTab = $derived(tabsStore.tabs.find(t => t.id === tabId));
  const currentSql = $derived(currentTab?.sql || initialSql || '');
  const activeConn = $derived(connectionStore.activeConnection);
  const sessionTxState = $derived(sessionStore.getState(activeConn?.id));
  const isProduction = $derived(activeConn?.environment === 'production');
  const sqlRisk = $derived(assessSqlRisk(currentSql));

  const lightSyntaxHighlight = HighlightStyle.define([
    { tag: tags.keyword, color: '#4338ca', fontWeight: '700' },
    { tag: tags.operatorKeyword, color: '#4338ca', fontWeight: '700' },
    { tag: tags.typeName, color: '#1d4ed8', fontWeight: '600' },
    { tag: tags.string, color: '#047857' },
    { tag: tags.number, color: '#b45309', fontWeight: '600' },
    { tag: tags.comment, color: '#64748b', fontStyle: 'italic' },
    { tag: tags.operator, color: '#4338ca' },
    { tag: tags.punctuation, color: '#334155' },
    { tag: tags.variableName, color: '#0f172a' },
    { tag: tags.propertyName, color: '#0f172a' },
  ]);

  function getEditorCustomTheme(isDark: boolean) {
    return EditorView.theme({
      '&': { 
        height: '100%', 
        fontSize: '13px', 
        backgroundColor: isDark ? '#020617' : '#ffffff',
        color: isDark ? '#f8fafc' : '#0f172a'
      },
      '.cm-scroller': { 
        overflow: 'auto', 
        fontFamily: 'JetBrains Mono, monospace' 
      },
      '.cm-gutters': { 
        backgroundColor: isDark ? '#090d16' : '#f8fafc', 
        borderRight: isDark ? '1px solid #1e293b' : '1px solid #e2e8f0', 
        color: isDark ? '#475569' : '#64748b' 
      },
      '.cm-activeLineGutter': { 
        backgroundColor: isDark ? '#1e293b' : '#e2e8f0' 
      },
      '.cm-activeLine': { 
        backgroundColor: isDark ? '#1e293b40' : '#f1f5f9' 
      },
      '.cm-selectionBackground, ::selection': {
        backgroundColor: isDark ? '#3b82f640' : '#bfdbfe !important'
      },
      '.cm-tooltip': {
        border: isDark ? '1px solid #334155' : '1px solid #cbd5e1',
        backgroundColor: isDark ? '#0f172a' : '#ffffff',
        borderRadius: '6px',
        boxShadow: isDark 
          ? '0 10px 25px -5px rgba(0, 0, 0, 0.5), 0 8px 10px -6px rgba(0, 0, 0, 0.5)' 
          : '0 10px 25px -5px rgba(0, 0, 0, 0.1), 0 8px 10px -6px rgba(0, 0, 0, 0.1)',
        zIndex: '100',
      },
      '.cm-tooltip.cm-tooltip-autocomplete': {
        fontFamily: 'JetBrains Mono, monospace',
      },
      '.cm-tooltip-autocomplete ul': {
        maxHeight: '260px',
        fontFamily: 'JetBrains Mono, monospace',
        fontSize: '12px',
        padding: '4px',
      },
      '.cm-tooltip-autocomplete ul li': {
        padding: '4px 8px',
        borderRadius: '4px',
        color: isDark ? '#e2e8f0' : '#1e293b',
        display: 'flex',
        alignItems: 'center',
        gap: '6px',
      },
      '.cm-tooltip-autocomplete ul li[aria-selected]': {
        backgroundColor: isDark ? '#1e293b' : '#e0e7ff',
        color: isDark ? '#ffffff' : '#312e81',
      },
      '.cm-completionLabel': {
        fontWeight: '500',
      },
      '.cm-completionMatchedText': {
        color: isDark ? '#818cf8' : '#4f46e5',
        fontWeight: '700',
        textDecoration: 'none',
      },
      '.cm-completionDetail': {
        fontStyle: 'normal',
        fontSize: '11px',
        color: isDark ? '#94a3b8' : '#64748b',
        marginLeft: 'auto',
      },
      '.cm-completionInfo': {
        backgroundColor: isDark ? '#020617' : '#f8fafc',
        border: isDark ? '1px solid #334155' : '1px solid #e2e8f0',
        padding: '8px',
        fontSize: '11px',
        borderRadius: '6px',
        maxWidth: '320px',
        whiteSpace: 'pre-wrap',
        color: isDark ? '#e2e8f0' : '#1e293b',
      }
    });
  }

  onMount(() => {
    const isDark = themeStore.resolvedTheme === 'dark';
    const runCurrentQuery = () => {
      tabsStore.runTabQuery(tabId);
      return true;
    };

    const startState = EditorState.create({
      doc: initialSql,
      extensions: [
        lineNumbers(),
        highlightActiveLineGutter(),
        highlightActiveLine(),
        history(),
        closeBrackets(),
        autocompletion({
          icons: true,
          defaultKeymap: true,
        }),
        languageCompartment.of(
          createSqlLanguageSupport(
            connectionStore.activeConnection?.driver,
            connectionStore.activeSchemaTree
          )
        ),
        themeCompartment.of([
          isDark ? oneDark : syntaxHighlighting(lightSyntaxHighlight),
          getEditorCustomTheme(isDark)
        ]),
        keymap.of([
          ...defaultKeymap,
          ...historyKeymap,
          ...closeBracketsKeymap,
          ...completionKeymap,
          {
            key: 'Mod-Enter',
            run: runCurrentQuery,
          },
          {
            key: 'Mod-e',
            run: () => {
              const doc = view?.state.doc.toString() || currentSql;
              if (doc.trim()) {
                explainStore.open(tabId, doc, true);
              }
              return true;
            },
          },
          {
            key: 'Mod-Shift-c',
            run: () => {
              sessionStore.commit(activeConn?.id, activeConn?.driver);
              return true;
            },
          },
          {
            key: 'Mod-Shift-r',
            run: () => {
              sessionStore.rollback(activeConn?.id, activeConn?.driver);
              return true;
            },
          },
        ]),
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            const current = tabsStore.tabs.find(t => t.id === tabId);
            if (current) {
              current.sql = update.state.doc.toString();
            }
          }
        }),
      ],
    });

    view = new EditorView({
      state: startState,
      parent: editorContainer,
    });

    const handleGlobalKeydown = (e: KeyboardEvent) => {
      const isMod = e.metaKey || e.ctrlKey;
      if (isMod && e.shiftKey && (e.key === 'c' || e.key === 'C')) {
        e.preventDefault();
        sessionStore.commit(activeConn?.id, activeConn?.driver);
      } else if (isMod && e.shiftKey && (e.key === 'r' || e.key === 'R')) {
        e.preventDefault();
        sessionStore.rollback(activeConn?.id, activeConn?.driver);
      }
    };
    window.addEventListener('keydown', handleGlobalKeydown);

    return () => {
      window.removeEventListener('keydown', handleGlobalKeydown);
      view?.destroy();
    };
  });

  $effect(() => {
    const isDark = themeStore.resolvedTheme === 'dark';
    if (view) {
      view.dispatch({
        effects: themeCompartment.reconfigure([
          isDark ? oneDark : syntaxHighlighting(lightSyntaxHighlight),
          getEditorCustomTheme(isDark)
        ])
      });
    }
  });

  $effect(() => {
    const driver = connectionStore.activeConnection?.driver;
    const schemaTree = connectionStore.activeSchemaTree;
    if (view) {
      view.dispatch({
        effects: languageCompartment.reconfigure(
          createSqlLanguageSupport(driver, schemaTree)
        )
      });
    }
  });

  $effect(() => {
    if (view && currentSql !== undefined) {
      const currentDoc = view.state.doc.toString();
      if (currentDoc !== currentSql) {
        view.dispatch({
          changes: { from: 0, to: currentDoc.length, insert: currentSql }
        });
      }
    }
  });
</script>

<div class="w-full h-full flex flex-col bg-surface-950 relative">
  <div bind:this={editorContainer} class="flex-1 overflow-hidden"></div>

  <!-- Inline Query Error Bar with Fix with AI Action -->
  {#if currentTab?.errorMessage}
    <div class="px-3 py-1.5 bg-rose-500/10 dark:bg-rose-950/40 border-t border-rose-500/30 flex items-center justify-between gap-3 text-xs animate-in fade-in duration-150 shrink-0">
      <div class="flex items-center gap-2 truncate min-w-0">
        <span class="w-2 h-2 rounded-full bg-rose-500 shrink-0"></span>
        <span class="font-bold text-[11px] text-rose-600 dark:text-rose-400 shrink-0 uppercase tracking-wider">Query Error:</span>
        <span class="font-mono text-rose-950 dark:text-rose-200 truncate text-[11px] font-medium select-text">{currentTab.errorMessage}</span>
      </div>

      <div class="flex items-center gap-2 shrink-0">
        <button
          type="button"
          onclick={() => aiStore.triggerFix(tabId, currentSql, currentTab.errorMessage || '')}
          class="flex items-center gap-1.5 px-2.5 py-1 bg-gradient-to-r from-violet-600 to-indigo-600 hover:from-violet-500 hover:to-indigo-500 text-white font-bold rounded text-[11px] shadow-sm transition-all cursor-pointer"
          title="Diagnose & Fix query error with AI Copilot"
        >
          <Sparkles size={12} class="animate-pulse" />
          <span>Fix with AI</span>
        </button>

        <button
          type="button"
          onclick={() => {
            if (currentTab) currentTab.errorMessage = undefined;
          }}
          class="text-rose-500 hover:text-rose-700 dark:hover:text-rose-300 p-0.5 rounded cursor-pointer"
          title="Dismiss error banner"
        >
          <X size={13} />
        </button>
      </div>
    </div>
  {/if}

  <!-- Bottom Editor Toolbar -->
  <div class="px-3 py-1.5 bg-surface-900 border-t border-slate-200 dark:border-slate-800 flex items-center justify-between text-xs select-none shrink-0">
    <div class="flex items-center gap-2 truncate">
      <button 
        type="button"
        onclick={() => tabsStore.runTabQuery(tabId)}
        class="flex items-center gap-1.5 px-2.5 py-1 bg-indigo-600 hover:bg-indigo-500 active:bg-indigo-700 text-white font-bold rounded text-xs shadow-xs transition-colors cursor-pointer"
        title="Execute Query (Cmd+Enter)"
      >
        <Play size={12} class="fill-current" />
        <span>Run</span>
      </button>

      <button 
        type="button"
        onclick={() => explainStore.open(tabId, currentSql, true)}
        class="flex items-center gap-1.5 px-2.5 py-1 bg-indigo-50/90 hover:bg-indigo-100/90 dark:bg-indigo-950/40 dark:hover:bg-indigo-900/50 text-indigo-700 dark:text-indigo-300 border border-indigo-200 dark:border-indigo-800/60 rounded text-xs font-semibold transition-colors cursor-pointer shadow-2xs"
        title="Visual Query EXPLAIN Execution Plan (Cmd+E / Ctrl+E)"
      >
        <Activity size={12} class="text-indigo-600 dark:text-indigo-400" />
        <span>Explain</span>
      </button>

      <!-- Transaction Controls -->
      <div class="h-3.5 w-px bg-slate-200 dark:bg-slate-700/80 mx-0.5"></div>

      <button
        type="button"
        onclick={() => sessionStore.toggleAutoCommit(activeConn?.id)}
        class="flex items-center gap-1.5 px-2 py-1 rounded text-xs font-semibold border transition-colors cursor-pointer {sessionTxState.autoCommit 
          ? 'bg-slate-100 dark:bg-surface-800 text-slate-700 dark:text-slate-300 border-slate-300/80 dark:border-slate-700 hover:bg-slate-200 dark:hover:bg-surface-700' 
          : 'bg-amber-500/15 text-amber-700 dark:text-amber-300 border-amber-500/30 hover:bg-amber-500/25'}"
        title={sessionTxState.autoCommit ? 'Auto-commit is ON. Click to switch to Manual Transaction mode.' : 'Manual Transaction mode active. Click to switch to Auto-commit.'}
      >
        {#if sessionTxState.autoCommit}
          <Zap size={11.5} class="text-emerald-500 fill-emerald-500" />
          <span>Auto-commit</span>
        {:else}
          <Lock size={11.5} class="text-amber-500" />
          <span>Manual Tx</span>
        {/if}
      </button>

      {#if !sessionTxState.autoCommit || sessionTxState.inTransaction}
        <div class="flex items-center gap-1 animate-in fade-in duration-150">
          <button
            type="button"
            onclick={() => sessionStore.commit(activeConn?.id, activeConn?.driver)}
            disabled={sessionStore.isOperating}
            class="flex items-center gap-1 px-2.5 py-1 bg-emerald-600 hover:bg-emerald-500 active:bg-emerald-700 text-white font-bold rounded text-xs shadow-xs transition-colors cursor-pointer disabled:opacity-50"
            title="Commit Transaction (Cmd+Shift+C / Ctrl+Shift+C)"
          >
            {#if sessionStore.isOperating}
              <Loader2 size={11} class="animate-spin" />
            {:else}
              <Check size={11} strokeWidth={3} />
            {/if}
            <span>Commit</span>
            {#if sessionTxState.uncommittedCount > 0}
              <span class="ml-0.5 px-1 py-0.2 text-[10px] bg-emerald-700 rounded-full font-mono font-bold leading-none">
                {sessionTxState.uncommittedCount}
              </span>
            {/if}
          </button>

          <button
            type="button"
            onclick={() => sessionStore.rollback(activeConn?.id, activeConn?.driver)}
            disabled={sessionStore.isOperating}
            class="flex items-center gap-1 px-2.5 py-1 bg-rose-600 hover:bg-rose-500 active:bg-rose-700 text-white font-bold rounded text-xs shadow-xs transition-colors cursor-pointer disabled:opacity-50"
            title="Rollback Transaction (Cmd+Shift+R / Ctrl+Shift+R)"
          >
            {#if sessionStore.isOperating}
              <Loader2 size={11} class="animate-spin" />
            {:else}
              <RotateCcw size={11} strokeWidth={2.5} />
            {/if}
            <span>Rollback</span>
          </button>
        </div>
      {/if}

      {#if isProduction}
        <div class="flex items-center gap-1.5 px-2 py-0.5 rounded text-[11px] font-bold bg-rose-500/15 text-rose-600 dark:text-rose-400 border border-rose-500/30">
          <ShieldAlert size={13} />
          <span>PRODUCTION GUARD ACTIVE</span>
        </div>

        {#if sqlRisk.isDangerous}
          <div class="flex items-center gap-1 px-2 py-0.5 rounded text-[10.5px] font-semibold bg-amber-500/15 text-amber-700 dark:text-amber-300 border border-amber-500/30 truncate">
            <AlertTriangle size={12} class="shrink-0 text-amber-500" />
            <span class="truncate">Destructive ({sqlRisk.destructiveCommands.join(', ')})</span>
          </div>
        {/if}
      {/if}

      <button 
        type="button"
        onclick={() => historyStore.toggleDrawer()}
        class="flex items-center gap-1.5 px-2.5 py-1 text-slate-700 dark:text-slate-300 hover:text-slate-900 dark:hover:text-slate-100 hover:bg-slate-100 dark:hover:bg-surface-800 rounded text-xs font-medium transition-colors cursor-pointer"
        title="Open Query History & Favorites (Cmd+H / Ctrl+H)"
      >
        <History size={13} class="text-slate-500 dark:text-slate-400" />
        <span>History</span>
      </button>

      <button 
        type="button"
        onclick={() => aiStore.openDrawer()}
        class="flex items-center gap-1.5 px-2.5 py-1 text-violet-700 dark:text-violet-300 hover:text-violet-800 dark:hover:text-violet-200 hover:bg-violet-50 dark:hover:bg-violet-950/40 rounded text-xs font-semibold transition-colors cursor-pointer"
        title="NL-to-SQL AI Assistant (Cmd+K / Ctrl+K)"
      >
        <Sparkles size={13} class="text-violet-600 dark:text-violet-400" />
        <span>Ask AI</span>
      </button>
    </div>

    <div class="flex items-center gap-3 text-[11px] text-slate-500 dark:text-slate-400 font-mono">
      <span>{activeConn?.driver?.toUpperCase() || 'SQL'}</span>
      <span>{activeConn?.name || 'Local'}</span>
    </div>
  </div>
</div>
