<script lang="ts">
  import { onMount } from 'svelte';
  import { EditorState, Compartment } from '@codemirror/state';
  import { EditorView, keymap, lineNumbers, highlightActiveLineGutter, highlightActiveLine } from '@codemirror/view';
  import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
  import { sql } from '@codemirror/lang-sql';
  import { syntaxHighlighting, HighlightStyle } from '@codemirror/language';
  import { tags } from '@lezer/highlight';
  import { oneDark } from '@codemirror/theme-one-dark';
  import { tabsStore } from '$lib/state/tabs.svelte';
  import { themeStore } from '$lib/state/theme.svelte';

  let { tabId, initialSql = '' }: { tabId: string; initialSql?: string } = $props();

  let editorContainer: HTMLDivElement;
  let view: EditorView | undefined = $state();
  const themeCompartment = new Compartment();

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
        sql(),
        themeCompartment.of([
          isDark ? oneDark : syntaxHighlighting(lightSyntaxHighlight),
          getEditorCustomTheme(isDark)
        ]),
        keymap.of([
          ...defaultKeymap,
          ...historyKeymap,
          {
            key: 'Mod-Enter',
            run: runCurrentQuery,
          },
        ]),
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            const currentTab = tabsStore.tabs.find(t => t.id === tabId);
            if (currentTab) {
              currentTab.sql = update.state.doc.toString();
            }
          }
        }),
      ],
    });

    view = new EditorView({
      state: startState,
      parent: editorContainer,
    });

    return () => {
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
    if (view && initialSql !== undefined) {
      const currentDoc = view.state.doc.toString();
      if (currentDoc !== initialSql) {
        view.dispatch({
          changes: { from: 0, to: currentDoc.length, insert: initialSql }
        });
      }
    }
  });
</script>

<div class="w-full h-full flex flex-col bg-surface-950">
  <div bind:this={editorContainer} class="flex-1 overflow-hidden"></div>
</div>
