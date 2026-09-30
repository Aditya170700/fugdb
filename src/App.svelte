<script lang="ts">
  import { onMount } from 'svelte';
  import Navbar from '$lib/components/layout/Navbar.svelte';
  import Statusbar from '$lib/components/layout/Statusbar.svelte';
  import ConnectionTree from '$lib/components/sidebar/ConnectionTree.svelte';
  import SqlEditor from '$lib/components/editor/SqlEditor.svelte';
  import DataGrid from '$lib/components/grid/DataGrid.svelte';
  import ConnectionModal from '$lib/components/connection/ConnectionModal.svelte';
  import ErdModal from '$lib/components/erd/ErdModal.svelte';
  import TransferModal from '$lib/components/transfer/TransferModal.svelte';
  import MockDataModal from '$lib/components/qa/MockDataModal.svelte';
  import { tabsStore } from '$lib/state/tabs.svelte';

  let isNewConnOpen = $state(false);
  let isErdOpen = $state(false);
  let isTransferOpen = $state(false);
  let isMockDataOpen = $state(false);

  const activeTab = $derived(tabsStore.activeTab);

  // Resizable state
  let sidebarWidth = $state(260);
  let editorHeightPercent = $state(38);
  let isDraggingSidebar = $state(false);
  let isDraggingEditor = $state(false);
  let sectionRef = $state<HTMLElement | null>(null);

  onMount(() => {
    try {
      const savedSidebar = localStorage.getItem('fugdb_sidebar_width');
      if (savedSidebar) {
        const val = parseInt(savedSidebar, 10);
        if (!isNaN(val)) sidebarWidth = Math.max(160, Math.min(600, val));
      }

      const savedEditor = localStorage.getItem('fugdb_editor_height_percent');
      if (savedEditor) {
        const val = parseFloat(savedEditor);
        if (!isNaN(val)) editorHeightPercent = Math.max(10, Math.min(88, val));
      }
    } catch {
      // Ignore localStorage access errors
    }
  });

  // Sidebar drag handler (Horizontal)
  function handleSidebarMouseDown(e: MouseEvent) {
    e.preventDefault();
    isDraggingSidebar = true;

    const onMouseMove = (moveEvent: MouseEvent) => {
      const maxWidth = Math.max(300, window.innerWidth * 0.55);
      const newWidth = Math.max(160, Math.min(maxWidth, moveEvent.clientX));
      sidebarWidth = newWidth;
    };

    const onMouseUp = () => {
      isDraggingSidebar = false;
      try {
        localStorage.setItem('fugdb_sidebar_width', sidebarWidth.toString());
      } catch {}
      window.removeEventListener('mousemove', onMouseMove);
      window.removeEventListener('mouseup', onMouseUp);
    };

    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('mouseup', onMouseUp);
  }

  // Editor vs Results split drag handler (Vertical)
  function handleEditorMouseDown(e: MouseEvent) {
    e.preventDefault();
    if (!sectionRef) return;
    isDraggingEditor = true;

    const onMouseMove = (moveEvent: MouseEvent) => {
      if (!sectionRef) return;
      const rect = sectionRef.getBoundingClientRect();
      const relativeY = moveEvent.clientY - rect.top;
      const percent = (relativeY / rect.height) * 100;
      editorHeightPercent = Math.max(10, Math.min(88, percent));
    };

    const onMouseUp = () => {
      isDraggingEditor = false;
      try {
        localStorage.setItem('fugdb_editor_height_percent', editorHeightPercent.toString());
      } catch {}
      window.removeEventListener('mousemove', onMouseMove);
      window.removeEventListener('mouseup', onMouseUp);
    };

    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('mouseup', onMouseUp);
  }

  function resetSidebarWidth() {
    sidebarWidth = 260;
    try {
      localStorage.setItem('fugdb_sidebar_width', '260');
    } catch {}
  }

  function resetEditorHeight() {
    editorHeightPercent = 38;
    try {
      localStorage.setItem('fugdb_editor_height_percent', '38');
    } catch {}
  }
</script>

<main class="w-screen h-screen flex flex-col bg-surface-950 text-slate-100 overflow-hidden font-sans select-none">
  <!-- Top Navigation Header -->
  <Navbar 
    onOpenNewConnection={() => isNewConnOpen = true}
    onOpenErd={() => isErdOpen = true}
    onOpenTransfer={() => isTransferOpen = true}
    onOpenMockData={() => isMockDataOpen = true}
  />

  <!-- Main Content Layout (Sidebar + Resizable Split Area) -->
  <div class="flex-1 flex overflow-hidden relative">
    <!-- Left Sidebar -->
    <ConnectionTree width={sidebarWidth} />

    <!-- 1. Vertical Resizer Handle (Sidebar <-> Main Area) -->
    <button 
      type="button"
      aria-label="Resize sidebar"
      onmousedown={handleSidebarMouseDown}
      ondblclick={resetSidebarWidth}
      class="w-1.5 hover:w-1.5 -ml-[1px] bg-slate-800/80 hover:bg-indigo-500 active:bg-indigo-400 cursor-col-resize transition-colors shrink-0 z-10 relative flex items-center justify-center group {isDraggingSidebar ? '!bg-indigo-500' : ''}"
      title="Drag to resize sidebar (Double click to reset)"
    >
      <span class="w-[2px] h-8 rounded-full bg-slate-600/40 group-hover:bg-indigo-200 transition-colors pointer-events-none {isDraggingSidebar ? '!bg-white' : ''}"></span>
    </button>

    <!-- Right Workspace Area -->
    <section bind:this={sectionRef} class="flex-1 flex flex-col overflow-hidden bg-surface-950 min-w-0">
      {#if activeTab}
        <!-- Top Split: SQL Editor -->
        <div style="height: {editorHeightPercent}%;" class="min-h-[60px] relative shrink-0 overflow-hidden">
          <SqlEditor tabId={activeTab.id} initialSql={activeTab.sql} />
        </div>

        <!-- 2. Horizontal Resizer Handle (SQL Editor <-> Result DataGrid) -->
        <button 
          type="button"
          aria-label="Resize editor and results panel"
          onmousedown={handleEditorMouseDown}
          ondblclick={resetEditorHeight}
          class="h-1.5 hover:h-1.5 bg-slate-800/80 hover:bg-indigo-500 active:bg-indigo-400 cursor-row-resize transition-colors shrink-0 z-10 relative flex items-center justify-center group {isDraggingEditor ? '!bg-indigo-500' : ''}"
          title="Drag to resize editor & result grid (Double click to reset)"
        >
          <span class="h-[2px] w-8 rounded-full bg-slate-600/40 group-hover:bg-indigo-200 transition-colors pointer-events-none {isDraggingEditor ? '!bg-white' : ''}"></span>
        </button>

        <!-- Bottom Split: Results Data Grid -->
        <div style="height: calc(100% - {editorHeightPercent}% - 6px);" class="min-h-[60px] overflow-hidden">
          <DataGrid result={activeTab.queryResult} errorMessage={activeTab.errorMessage} />
        </div>
      {:else}
        <div class="flex-1 flex items-center justify-center text-slate-500 text-sm">
          No active query tab. Click + in topbar to open a new tab.
        </div>
      {/if}
    </section>

    <!-- Transparent drag blocker overlay during active mouse drag -->
    {#if isDraggingSidebar || isDraggingEditor}
      <div 
        class="fixed inset-0 z-50 select-none {isDraggingSidebar ? 'cursor-col-resize' : 'cursor-row-resize'}"
        role="presentation"
      ></div>
    {/if}
  </div>

  <!-- Bottom Status Bar -->
  <Statusbar />

  <!-- Modals -->
  <ConnectionModal isOpen={isNewConnOpen} onClose={() => isNewConnOpen = false} />
  <ErdModal isOpen={isErdOpen} onClose={() => isErdOpen = false} />
  <TransferModal isOpen={isTransferOpen} onClose={() => isTransferOpen = false} />
  <MockDataModal isOpen={isMockDataOpen} onClose={() => isMockDataOpen = false} />
</main>
