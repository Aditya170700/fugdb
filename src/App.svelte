<script lang="ts">
  import Navbar from '$lib/components/layout/Navbar.svelte';
  import Statusbar from '$lib/components/layout/Statusbar.svelte';
  import ConnectionTree from '$lib/components/sidebar/ConnectionTree.svelte';
  import SqlEditor from '$lib/components/editor/SqlEditor.svelte';
  import DataGrid from '$lib/components/grid/DataGrid.svelte';
  import ErdModal from '$lib/components/erd/ErdModal.svelte';
  import TransferModal from '$lib/components/transfer/TransferModal.svelte';
  import MockDataModal from '$lib/components/qa/MockDataModal.svelte';
  import { tabsStore } from '$lib/state/tabs.svelte';

  let isErdOpen = $state(false);
  let isTransferOpen = $state(false);
  let isMockDataOpen = $state(false);

  const activeTab = $derived(tabsStore.activeTab);
</script>

<main class="w-screen h-screen flex flex-col bg-surface-950 text-slate-100 overflow-hidden font-sans">
  <!-- Top Navigation Header -->
  <Navbar 
    onOpenErd={() => isErdOpen = true}
    onOpenTransfer={() => isTransferOpen = true}
    onOpenMockData={() => isMockDataOpen = true}
  />

  <!-- Main Content Layout (Sidebar + Split Editor/Grid Area) -->
  <div class="flex-1 flex overflow-hidden">
    <ConnectionTree />

    <section class="flex-1 flex flex-col overflow-hidden bg-surface-950">
      {#if activeTab}
        <!-- Top Split: SQL Editor -->
        <div class="h-[40%] min-h-[140px] border-b border-slate-800 relative">
          <SqlEditor tabId={activeTab.id} initialSql={activeTab.sql} />
        </div>

        <!-- Bottom Split: Results Data Grid -->
        <div class="flex-1 overflow-hidden">
          <DataGrid result={activeTab.queryResult} />
        </div>
      {:else}
        <div class="flex-1 flex items-center justify-center text-slate-500 text-sm">
          No active query tab. Click + in topbar to open a new tab.
        </div>
      {/if}
    </section>
  </div>

  <!-- Bottom Status Bar -->
  <Statusbar />

  <!-- Modals -->
  <ErdModal isOpen={isErdOpen} onClose={() => isErdOpen = false} />
  <TransferModal isOpen={isTransferOpen} onClose={() => isTransferOpen = false} />
  <MockDataModal isOpen={isMockDataOpen} onClose={() => isMockDataOpen = false} />
</main>
