<script lang="ts">
  import { onMount } from 'svelte';
  import { EditorState, Compartment } from '@codemirror/state';
  import { EditorView, keymap, lineNumbers, highlightActiveLineGutter, highlightActiveLine } from '@codemirror/view';
  import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
  import { autocompletion, completionKeymap, closeBrackets, closeBracketsKeymap } from '@codemirror/autocomplete';
  import { syntaxHighlighting, HighlightStyle } from '@codemirror/language';
  import { tags } from '@lezer/highlight';
  import { oneDark } from '@codemirror/theme-one-dark';
  import { tabsStore } from '$lib/state/tabs.svelte';
  import { themeStore } from '$lib/state/theme.svelte';
  import { connectionStore } from '$lib/state/connection.svelte';
  import { createSqlLanguageSupport } from './sqlCompletion';

  let { tabId, initialSql = '' }: { tabId: string; initialSql?: string } = $props();

  let editorContainer: HTMLDivElement;
  let view: EditorView | undefined = $state();
  const themeCompartment = new Compartment();
  const languageCompartment = new Compartment();

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
      '.cm-tooltip-autocomplete': {
        '& > ul': {
          maxHeight: '260px',
          fontFamily: 'JetBrains Mono, monospace',
          fontSize: '12px',
          padding: '4px',
        },
        '& > ul > li': {
          padding: '4px 8px',
          borderRadius: '4px',
          color: isDark ? '#e2e8f0' : '#1e293b',
          display: 'flex',
          alignItems: 'center',
          gap: '6px',
        },
        '& > ul > li[aria-selected]': {
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
