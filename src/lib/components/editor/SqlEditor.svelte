<script lang="ts">
  import { onMount } from 'svelte';
  import { EditorState } from '@codemirror/state';
  import { EditorView, keymap, lineNumbers, highlightActiveLineGutter, highlightActiveLine } from '@codemirror/view';
  import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
  import { sql } from '@codemirror/lang-sql';
  import { oneDark } from '@codemirror/theme-one-dark';
  import { tabsStore } from '$lib/state/tabs.svelte';

  let { tabId, initialSql = '' }: { tabId: string; initialSql?: string } = $props();

  let editorContainer: HTMLDivElement;
  let view: EditorView;

  onMount(() => {
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
        oneDark,
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
        EditorView.theme({
          '&': { height: '100%', fontSize: '13px', backgroundColor: '#090d16' },
          '.cm-scroller': { overflow: 'auto', fontFamily: 'JetBrains Mono, monospace' },
          '.cm-gutters': { backgroundColor: '#090d16', borderRight: '1px solid #1e293b', color: '#475569' },
          '.cm-activeLineGutter': { backgroundColor: '#1e293b' },
          '.cm-activeLine': { backgroundColor: '#1e293b40' },
        }),
      ],
    });

    view = new EditorView({
      state: startState,
      parent: editorContainer,
    });

    return () => {
      view.destroy();
    };
  });
</script>

<div class="w-full h-full flex flex-col bg-[#090d16]">
  <div bind:this={editorContainer} class="flex-1 overflow-hidden"></div>
</div>
