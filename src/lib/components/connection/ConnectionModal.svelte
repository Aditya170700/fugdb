<script lang="ts">
  import type { ConnectionConfig, DriverType, Environment } from '$lib/api/types';
  import { connectionStore } from '$lib/state/connection.svelte';
  import { api } from '$lib/api/client';
  import { open } from '@tauri-apps/plugin-dialog';
  import { 
    Database, 
    X, 
    ShieldAlert, 
    Key, 
    Terminal, 
    Lock, 
    Eye, 
    EyeOff, 
    CheckCircle2, 
    AlertCircle, 
    FolderOpen,
    Search,
    ChevronDown,
    Check,
    KeyRound,
    Shield,
    Zap,
    Server
  } from 'lucide-svelte';

  let { isOpen, onClose }: { isOpen: boolean; onClose: () => void } = $props();

  interface EngineOption {
    id: DriverType;
    label: string;
    icon: string;
    category: string;
    defaultPort?: number;
    description: string;
    isSupported: boolean;
  }

  const allEngines: EngineOption[] = [
    { id: 'postgres', label: 'PostgreSQL', icon: '🐘', category: 'Relational (SQL)', defaultPort: 15432, description: 'Advanced open-source relational database', isSupported: true },
    { id: 'mysql', label: 'MySQL', icon: '🐬', category: 'Relational (SQL)', defaultPort: 3306, description: 'The world\'s most popular open-source database', isSupported: true },
    { id: 'mysql', label: 'MariaDB', icon: '🦭', category: 'Relational (SQL)', defaultPort: 3306, description: 'High-performance MySQL drop-in alternative', isSupported: true },
    { id: 'sqlite', label: 'SQLite', icon: '🪶', category: 'Embedded / Local', description: 'Self-contained, serverless zero-configuration DB', isSupported: true },
    { id: 'duckdb', label: 'DuckDB', icon: '🦆', category: 'Embedded / Analytics', description: 'Fast in-process analytical SQL database', isSupported: false },
    { id: 'mssql', label: 'Microsoft SQL Server', icon: '🪟', category: 'Enterprise (SQL)', defaultPort: 1433, description: 'Enterprise relational database from Microsoft', isSupported: true },
    { id: 'postgres', label: 'CockroachDB', icon: '🪳', category: 'Distributed SQL', defaultPort: 26257, description: 'Cloud-native distributed PostgreSQL-wire DB', isSupported: true },
    { id: 'postgres', label: 'TimescaleDB', icon: '⏱️', category: 'Time-Series', defaultPort: 5432, description: 'Time-series database built on PostgreSQL', isSupported: true },
  ];

  let isEngineDropdownOpen = $state(false);
  let engineSearchQuery = $state('');

  let activeTab = $state<'general' | 'ssh' | 'ssl'>('general');
  let selectedDriver = $state<DriverType>('postgres');
  let selectedEngineLabel = $state('PostgreSQL');
  let selectedEngineIcon = $state('🐘');

  let name = $state('My PostgreSQL');
  let environment = $state<Environment>('dev');
  let host = $state('localhost');
  let port = $state(15432);
  let database = $state('fugdb_test');
  let username = $state('fugdb_user');
  let password = $state('fugdb_password');
  let filePath = $state('');
  let showPassword = $state(false);
  let savePasswordToKeyring = $state(true);

  // SSH Tunnel Fields
  let useSsh = $state(false);
  let sshHost = $state('');
  let sshPort = $state(22);
  let sshUser = $state('ubuntu');
  let sshAuthType = $state<'key' | 'password' | 'agent'>('key');
  let sshPassword = $state('');
  let sshKeyPath = $state('~/.ssh/id_rsa');
  let sshKeyPassphrase = $state('');
  let showSshPassword = $state(false);
  let showSshPassphrase = $state(false);
  let isTestingSsh = $state(false);

  // SSL Field
  let sslMode = $state<'disable' | 'prefer' | 'require'>('prefer');

  // Test Connection State
  let isTesting = $state(false);
  let testResult = $state<{ success: boolean; message: string; latencyMs?: number } | null>(null);
  let isSaving = $state(false);

  const filteredEngines = $derived.by(() => {
    if (!engineSearchQuery.trim()) return allEngines;
    const q = engineSearchQuery.toLowerCase();
    return allEngines.filter(e => 
      e.label.toLowerCase().includes(q) || 
      e.category.toLowerCase().includes(q) ||
      e.description.toLowerCase().includes(q)
    );
  });

  function selectEngine(engine: EngineOption) {
    if (!engine.isSupported) return;
    selectedDriver = engine.id;
    selectedEngineLabel = engine.label;
    selectedEngineIcon = engine.icon;
    isEngineDropdownOpen = false;
    engineSearchQuery = '';

    if (engine.id === 'postgres') {
      port = engine.defaultPort || 15432;
      database = 'fugdb_test';
      username = 'fugdb_user';
      name = `${engine.label} Connection`;
    } else if (engine.id === 'mysql') {
      port = engine.defaultPort || 3306;
      database = 'fugdb_test';
      username = 'fugdb_user';
      name = `${engine.label} Connection`;
    } else if (engine.id === 'sqlite' || engine.id === 'duckdb') {
      name = `${engine.label} Database`;
      filePath = engine.id === 'duckdb' ? 'local.duckdb' : 'local.sqlite';
    } else if (engine.id === 'mssql') {
      port = 1433;
      database = 'master';
      username = 'sa';
      name = 'MSSQL Connection';
    }
  }

  async function handleBrowseSshKey() {
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        title: 'Select SSH Private Key'
      });
      if (typeof selected === 'string') {
        sshKeyPath = selected;
      }
    } catch (e) {
      console.warn('File picker dismissed or unavailable:', e);
    }
  }

  async function handleBrowseDbFile() {
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        title: 'Select SQLite Database File',
        filters: [{ name: 'SQLite Database', extensions: ['db', 'sqlite', 'sqlite3', 'duckdb', '*'] }]
      });
      if (typeof selected === 'string') {
        filePath = selected;
      }
    } catch (e) {
      console.warn('File picker dismissed or unavailable:', e);
    }
  }

  function getFormConfig(): ConnectionConfig {
    return {
      id: `conn-${Date.now()}`,
      name: name.trim() || `${selectedEngineLabel} Database`,
      driver: selectedDriver,
      environment,
      host: selectedDriver !== 'sqlite' && selectedDriver !== 'duckdb' ? host : undefined,
      port: selectedDriver !== 'sqlite' && selectedDriver !== 'duckdb' ? port : undefined,
      database: selectedDriver !== 'sqlite' && selectedDriver !== 'duckdb' ? database : undefined,
      username: selectedDriver !== 'sqlite' && selectedDriver !== 'duckdb' ? username : undefined,
      password: selectedDriver !== 'sqlite' && selectedDriver !== 'duckdb' ? password : undefined,
      filePath: selectedDriver === 'sqlite' || selectedDriver === 'duckdb' ? filePath : undefined,
      useSsh,
      sshHost: useSsh ? sshHost : undefined,
      sshPort: useSsh ? sshPort : undefined,
      sshUser: useSsh ? sshUser : undefined,
      sshAuthType: useSsh ? sshAuthType : undefined,
      sshPassword: useSsh && sshAuthType === 'password' ? sshPassword : undefined,
      sshKeyPath: useSsh && sshAuthType === 'key' ? sshKeyPath : undefined,
      sshKeyPassphrase: useSsh && sshAuthType === 'key' ? sshKeyPassphrase : undefined,
      sslMode,
      savePasswordToKeyring,
      useKeyring: savePasswordToKeyring,
    };
  }

  async function handleTest() {
    isTesting = true;
    testResult = null;
    try {
      const config = getFormConfig();
      const res = await api.testConnection(config);
      testResult = res;
    } catch (err: any) {
      testResult = {
        success: false,
        message: err?.toString() || 'Connection failed.',
      };
    } finally {
      isTesting = false;
    }
  }

  async function handleTestSsh() {
    isTestingSsh = true;
    testResult = null;
    try {
      const config = getFormConfig();
      const res = await api.testSshTunnel(config);
      testResult = res;
    } catch (err: any) {
      testResult = {
        success: false,
        message: err?.toString() || 'SSH handshake failed.',
      };
    } finally {
      isTestingSsh = false;
    }
  }

  async function handleSaveAndConnect() {
    isSaving = true;
    try {
      const config = getFormConfig();
      await api.connect(config);
      connectionStore.addConnection(config);
      onClose();
    } catch (err: any) {
      testResult = {
        success: false,
        message: `Failed to connect: ${err?.toString() || err}`,
      };
    } finally {
      isSaving = false;
    }
  }
</script>

{#if isOpen}
  <div class="fixed inset-0 bg-black/70 backdrop-blur-sm z-50 flex items-center justify-center p-4">
    <div class="bg-surface-900 border border-slate-200 dark:border-slate-700 w-full max-w-2xl rounded-xl shadow-2xl overflow-visible flex flex-col">
      <!-- Header -->
      <div class="px-5 py-4 border-b border-slate-200 dark:border-slate-800 flex items-center justify-between">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 rounded-lg bg-indigo-600 text-white flex items-center justify-center shadow-md shadow-indigo-600/30">
            <Database size={18} />
          </div>
          <div>
            <h3 class="font-bold text-sm text-slate-900 dark:text-slate-100">Create New Connection</h3>
            <p class="text-[11px] text-slate-500 dark:text-slate-400">Configure PostgreSQL, MySQL, SQLite, MariaDB, or MSSQL database</p>
          </div>
        </div>
        <button onclick={onClose} class="text-slate-500 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-100 hover:bg-surface-800 p-1.5 rounded-lg transition-colors" title="Close">
          <X size={16} />
        </button>
      </div>

      <!-- Test Connection Result Alert Banner (Above Database Engine) -->
      {#if testResult}
        <div class="mx-6 mt-4 p-3 rounded-xl flex items-start justify-between gap-3 text-xs {testResult.success ? 'bg-emerald-500/10 dark:bg-emerald-500/15 border border-emerald-500/30 dark:border-emerald-500/40 text-emerald-950 dark:text-emerald-200' : 'bg-rose-500/10 dark:bg-rose-500/15 border border-rose-500/30 dark:border-rose-500/40 text-rose-950 dark:text-rose-200'}">
          <div class="flex items-start gap-2.5 min-w-0">
            {#if testResult.success}
              <CheckCircle2 size={17} class="shrink-0 text-emerald-600 dark:text-emerald-400 mt-0.5" />
              <div class="space-y-0.5">
                <div class="flex items-center gap-2 flex-wrap">
                  <span class="font-bold text-emerald-900 dark:text-emerald-300">Connection Successful!</span>
                  {#if testResult.latencyMs !== undefined}
                    <span class="px-1.5 py-0.5 rounded bg-emerald-500/20 text-[10px] font-mono font-semibold text-emerald-800 dark:text-emerald-300 border border-emerald-500/40">
                      ⚡ {testResult.latencyMs}ms latency
                    </span>
                  {/if}
                </div>
                <p class="text-[11px] text-emerald-800 dark:text-emerald-300/90 leading-snug">{testResult.message}</p>
              </div>
            {:else}
              <AlertCircle size={17} class="shrink-0 text-rose-600 dark:text-rose-400 mt-0.5" />
              <div class="space-y-0.5 min-w-0">
                <span class="font-bold text-rose-900 dark:text-rose-300 block">Connection Failed</span>
                <p class="text-[11px] text-rose-800 dark:text-rose-200/90 leading-snug break-words whitespace-pre-wrap">{testResult.message}</p>
              </div>
            {/if}
          </div>
          <button 
            type="button" 
            onclick={() => testResult = null}
            class="text-slate-400 hover:text-slate-700 dark:hover:text-slate-100 hover:bg-slate-200 dark:hover:bg-surface-800 p-1 rounded-md transition-colors shrink-0"
            title="Dismiss"
          >
            <X size={14} />
          </button>
        </div>
      {/if}

      <!-- Searchable Engine Combobox -->
      <div class="px-6 pt-4 pb-2 relative">
        <label for="engine-combobox-btn" class="text-[10px] font-semibold uppercase text-slate-500 dark:text-slate-400 block mb-1.5">
          Database Engine
        </label>
        
        <!-- Trigger Button -->
        <button 
          id="engine-combobox-btn"
          type="button"
          onclick={() => isEngineDropdownOpen = !isEngineDropdownOpen}
          class="w-full bg-surface-950 border border-slate-200 dark:border-slate-700 hover:border-indigo-500 rounded-lg px-3.5 py-2.5 flex items-center justify-between text-xs text-slate-900 dark:text-slate-100 transition-colors shadow-sm"
        >
          <div class="flex items-center gap-2.5">
            <span class="text-lg">{selectedEngineIcon}</span>
            <span class="font-bold text-slate-900 dark:text-slate-100">{selectedEngineLabel}</span>
            <span class="text-[10px] px-2 py-0.5 rounded bg-surface-800 text-slate-600 dark:text-slate-400 font-normal">
              {allEngines.find(e => e.label === selectedEngineLabel)?.category || 'SQL'}
            </span>
          </div>
          <div class="flex items-center gap-1.5 text-slate-500 dark:text-slate-400">
            <span class="text-[11px]">Change</span>
            <ChevronDown size={14} class="transition-transform duration-200 {isEngineDropdownOpen ? 'rotate-180' : ''}" />
          </div>
        </button>

        <!-- Searchable Dropdown Popover -->
        {#if isEngineDropdownOpen}
          <div class="absolute left-6 right-6 top-[72px] z-50 bg-surface-900 border border-slate-200 dark:border-slate-700 rounded-xl shadow-2xl p-2.5 flex flex-col gap-2 max-h-72">
            <!-- Search input inside dropdown -->
            <div class="relative">
              <Search size={13} class="absolute left-3 top-2.5 text-slate-400 dark:text-slate-500" />
              <input 
                type="text" 
                bind:value={engineSearchQuery}
                placeholder="Search database engine (e.g. postgres, mysql, sqlite)..." 
                class="w-full bg-surface-950 border border-slate-200 dark:border-slate-700/80 text-xs rounded-lg pl-8 pr-3 py-2 text-slate-900 dark:text-slate-100 focus:outline-none focus:border-indigo-500 placeholder:text-slate-400"
              />
            </div>

            <!-- List of engines -->
            <div class="overflow-y-auto space-y-1 pr-1 max-h-52">
              {#if filteredEngines.length === 0}
                <div class="p-4 text-center text-xs text-slate-500">No database engine found.</div>
              {:else}
                {#each filteredEngines as engine}
                  <button 
                    type="button"
                    onclick={() => selectEngine(engine)}
                    disabled={!engine.isSupported}
                    class="w-full flex items-center justify-between p-2 rounded-lg text-left transition-colors {selectedEngineLabel === engine.label ? 'bg-indigo-50 dark:bg-indigo-600/20 border border-indigo-300 dark:border-indigo-500/40 text-indigo-900 dark:text-indigo-200' : engine.isSupported ? 'hover:bg-surface-800 text-slate-800 dark:text-slate-100' : 'opacity-50 cursor-not-allowed hover:bg-transparent text-slate-500 dark:text-slate-400'}"
                  >
                    <div class="flex items-center gap-2.5 truncate">
                      <span class="text-base shrink-0">{engine.icon}</span>
                      <div class="truncate">
                        <div class="font-bold text-xs flex items-center gap-1.5">
                          <span class={selectedEngineLabel === engine.label ? 'text-indigo-700 dark:text-indigo-200' : 'text-slate-800 dark:text-slate-100'}>{engine.label}</span>
                          <span class="text-[10px] font-normal {selectedEngineLabel === engine.label ? 'text-indigo-600/80 dark:text-indigo-300/80' : 'text-slate-500 dark:text-slate-400'}">({engine.category})</span>
                        </div>
                        <p class="text-[10px] {selectedEngineLabel === engine.label ? 'text-indigo-800/80 dark:text-indigo-200/80' : 'text-slate-500 dark:text-slate-400'} truncate">{engine.description}</p>
                      </div>
                    </div>

                    {#if !engine.isSupported}
                      <span class="text-[9px] px-1.5 py-0.5 rounded bg-amber-500/15 text-amber-700 dark:text-amber-400 font-semibold border border-amber-500/30 shrink-0 ml-2">
                        Coming Soon
                      </span>
                    {:else if selectedEngineLabel === engine.label}
                      <Check size={14} class="text-indigo-600 dark:text-indigo-400 shrink-0 ml-2" />
                    {/if}
                  </button>
                {/each}
              {/if}
            </div>
          </div>
        {/if}
      </div>

      <!-- Tabs Navigation -->
      <div class="flex border-b border-slate-200 dark:border-slate-800 px-6 gap-6 text-xs font-medium">
        <button 
          type="button"
          onclick={() => activeTab = 'general'}
          class="py-2.5 border-b-2 transition-colors {activeTab === 'general' ? 'border-indigo-500 text-indigo-600 dark:text-indigo-400 font-bold' : 'border-transparent text-slate-500 dark:text-slate-400 hover:text-slate-800 dark:hover:text-slate-200'}"
        >
          General Settings
        </button>
        <button 
          type="button"
          onclick={() => activeTab = 'ssh'}
          class="py-2.5 border-b-2 transition-colors flex items-center gap-1.5 {activeTab === 'ssh' ? 'border-indigo-500 text-indigo-600 dark:text-indigo-400 font-bold' : 'border-transparent text-slate-500 dark:text-slate-400 hover:text-slate-800 dark:hover:text-slate-200'}"
        >
          <Terminal size={13} />
          <span>SSH Tunnel</span>
          {#if useSsh}<span class="w-1.5 h-1.5 rounded-full bg-emerald-500"></span>{/if}
        </button>
        <button 
          type="button"
          onclick={() => activeTab = 'ssl'}
          class="py-2.5 border-b-2 transition-colors flex items-center gap-1.5 {activeTab === 'ssl' ? 'border-indigo-500 text-indigo-600 dark:text-indigo-400 font-bold' : 'border-transparent text-slate-500 dark:text-slate-400 hover:text-slate-800 dark:hover:text-slate-200'}"
        >
          <Lock size={13} />
          <span>SSL / TLS</span>
        </button>
      </div>

      <!-- Tab Content Area -->
      <div class="p-6 space-y-4 text-xs min-h-[420px] max-h-[540px] overflow-y-auto">
        {#if activeTab === 'general'}
          <!-- Name & Environment -->
          <div class="space-y-3">
            <div class="space-y-1">
              <label for="conn-name" class="font-semibold text-slate-700 dark:text-slate-300 text-[11px]">Connection Name</label>
              <input id="conn-name" bind:value={name} placeholder="e.g. Local PostgreSQL" class="w-full bg-surface-950 border border-slate-200 dark:border-slate-800 rounded-lg px-3 py-2 text-slate-900 dark:text-slate-100 focus:outline-none focus:border-indigo-500 shadow-sm" />
            </div>

            <div class="space-y-1.5">
              <span class="font-semibold text-slate-800 dark:text-slate-100 text-[11px] block">Environment & Safety Level</span>
              <div class="grid grid-cols-3 gap-2">
                <button
                  type="button"
                  onclick={() => environment = 'dev'}
                  class="flex items-center justify-center gap-2 py-2 px-3 rounded-lg border text-xs font-bold transition-all {environment === 'dev' ? 'bg-emerald-500/15 border-emerald-500 text-emerald-800 dark:text-emerald-300 shadow-sm shadow-emerald-500/20' : 'bg-surface-950/70 border-slate-200 dark:border-slate-800 text-slate-600 dark:text-slate-400 hover:border-slate-300 dark:hover:border-slate-700 hover:text-slate-900 dark:hover:text-slate-100'}"
                >
                  <span class="w-2 h-2 rounded-full bg-emerald-500 {environment === 'dev' ? 'animate-pulse' : ''}"></span>
                  <span>Development</span>
                </button>

                <button
                  type="button"
                  onclick={() => environment = 'staging'}
                  class="flex items-center justify-center gap-2 py-2 px-3 rounded-lg border text-xs font-bold transition-all {environment === 'staging' ? 'bg-amber-500/15 border-amber-500 text-amber-800 dark:text-amber-300 shadow-sm shadow-amber-500/20' : 'bg-surface-950/70 border-slate-200 dark:border-slate-800 text-slate-600 dark:text-slate-400 hover:border-slate-300 dark:hover:border-slate-700 hover:text-slate-900 dark:hover:text-slate-100'}"
                >
                  <span class="w-2 h-2 rounded-full bg-amber-500 {environment === 'staging' ? 'animate-pulse' : ''}"></span>
                  <span>Staging</span>
                </button>

                <button
                  type="button"
                  onclick={() => environment = 'production'}
                  class="flex items-center justify-center gap-2 py-2 px-3 rounded-lg border text-xs font-bold transition-all {environment === 'production' ? 'bg-rose-500/15 border-rose-500 text-rose-800 dark:text-rose-300 shadow-sm shadow-rose-500/20' : 'bg-surface-950/70 border-slate-200 dark:border-slate-800 text-slate-600 dark:text-slate-400 hover:border-slate-300 dark:hover:border-slate-700 hover:text-slate-900 dark:hover:text-slate-100'}"
                >
                  <span class="w-2 h-2 rounded-full bg-rose-500 {environment === 'production' ? 'animate-pulse' : ''}"></span>
                  <span>Production</span>
                </button>
              </div>
            </div>
          </div>

          {#if environment === 'production'}
            <div class="p-2.5 bg-rose-500/10 border border-rose-500/30 rounded-lg flex items-center gap-2 text-rose-700 dark:text-rose-300 text-[11px]">
              <ShieldAlert size={15} class="shrink-0 text-rose-600 dark:text-rose-400" />
              <span>Production Safety Guard will prompt before executing destructive queries (DROP/TRUNCATE/DELETE).</span>
            </div>
          {/if}

          {#if selectedDriver === 'sqlite' || selectedDriver === 'duckdb'}
            <!-- File-based DB -->
            <div class="space-y-1">
              <label for="conn-filepath" class="font-semibold text-slate-700 dark:text-slate-300 text-[11px]">Database File Path</label>
              <div class="flex gap-2">
                <input id="conn-filepath" bind:value={filePath} placeholder="/path/to/database.sqlite" class="flex-1 bg-surface-950 border border-slate-200 dark:border-slate-800 rounded-md px-3 py-1.5 text-slate-900 dark:text-slate-100 font-mono text-xs focus:outline-none focus:border-indigo-500" />
                <button 
                  type="button" 
                  onclick={handleBrowseDbFile}
                  class="px-3 bg-surface-800 hover:bg-surface-700 text-slate-700 dark:text-slate-300 rounded-md flex items-center gap-1.5 text-xs border border-slate-200 dark:border-slate-700 cursor-pointer"
                >
                  <FolderOpen size={14} /> Browse
                </button>
              </div>
            </div>
          {:else}
            <!-- Host & Port -->
            <div class="grid grid-cols-4 gap-3">
              <div class="col-span-3 space-y-1">
                <label for="conn-host" class="font-semibold text-slate-700 dark:text-slate-300 text-[11px]">Host / IP Address</label>
                <input id="conn-host" bind:value={host} placeholder="localhost" class="w-full bg-surface-950 border border-slate-200 dark:border-slate-800 rounded-md px-3 py-1.5 text-slate-900 dark:text-slate-100 focus:outline-none focus:border-indigo-500" />
              </div>
              <div class="space-y-1">
                <label for="conn-port" class="font-semibold text-slate-700 dark:text-slate-300 text-[11px]">Port</label>
                <input id="conn-port" type="number" bind:value={port} class="w-full bg-surface-950 border border-slate-200 dark:border-slate-800 rounded-md px-3 py-1.5 text-slate-900 dark:text-slate-100 focus:outline-none focus:border-indigo-500" />
              </div>
            </div>

            <!-- Database Name -->
            <div class="space-y-1">
              <label for="conn-db" class="font-semibold text-slate-700 dark:text-slate-300 text-[11px]">Database</label>
              <input id="conn-db" bind:value={database} placeholder="Database Name" class="w-full bg-surface-950 border border-slate-200 dark:border-slate-800 rounded-md px-3 py-1.5 text-slate-900 dark:text-slate-100 focus:outline-none focus:border-indigo-500" />
            </div>

            <!-- Username & Password -->
            <div class="grid grid-cols-2 gap-3">
              <div class="space-y-1">
                <label for="conn-user" class="font-semibold text-slate-700 dark:text-slate-300 text-[11px]">Username</label>
                <input id="conn-user" bind:value={username} placeholder="postgres / root" class="w-full bg-surface-950 border border-slate-200 dark:border-slate-800 rounded-md px-3 py-1.5 text-slate-900 dark:text-slate-100 focus:outline-none focus:border-indigo-500" />
              </div>
              <div class="space-y-1">
                <label for="conn-pass" class="font-semibold text-slate-700 dark:text-slate-300 text-[11px]">Password</label>
                <div class="relative">
                  <input 
                    id="conn-pass"
                    type={showPassword ? 'text' : 'password'} 
                    bind:value={password} 
                    placeholder="••••••••" 
                    class="w-full bg-surface-950 border border-slate-200 dark:border-slate-800 rounded-md pl-3 pr-8 py-1.5 text-slate-900 dark:text-slate-100 focus:outline-none focus:border-indigo-500" 
                  />
                  <button 
                    type="button"
                    onclick={() => showPassword = !showPassword}
                    class="absolute right-2.5 top-2 text-slate-400 dark:text-slate-500 hover:text-slate-700 dark:hover:text-slate-300 cursor-pointer"
                  >
                    {#if showPassword}<EyeOff size={14} />{:else}<Eye size={14} />{/if}
                  </button>
                </div>
              </div>
            </div>

            <!-- OS Keyring Credential Security Option -->
            <div class="pt-1 flex items-center justify-between">
              <label class="flex items-center gap-2 cursor-pointer select-none text-[11px] text-slate-700 dark:text-slate-300">
                <input 
                  type="checkbox" 
                  bind:checked={savePasswordToKeyring} 
                  class="rounded border-slate-300 dark:border-slate-700 text-indigo-600 focus:ring-indigo-500 cursor-pointer"
                />
                <span class="flex items-center gap-1.5 font-medium">
                  <Lock size={12} class="text-emerald-500 shrink-0" />
                  <span>Store password securely in OS Keychain / Keyring</span>
                </span>
              </label>
              <span class="text-[10px] text-emerald-600 dark:text-emerald-400 font-semibold uppercase tracking-wider bg-emerald-500/10 px-1.5 py-0.5 rounded border border-emerald-500/20">
                Protected
              </span>
            </div>
          {/if}

        {:else if activeTab === 'ssh'}
          <!-- SSH Tab -->
          <div class="space-y-4">
            <div class="flex items-center justify-between p-3 rounded-xl border border-indigo-500/30 bg-indigo-500/10 dark:bg-indigo-500/15">
              <label class="flex items-center gap-2.5 cursor-pointer select-none">
                <input 
                  type="checkbox" 
                  bind:checked={useSsh} 
                  class="w-4 h-4 rounded border-slate-300 dark:border-slate-700 text-indigo-600 focus:ring-indigo-500 cursor-pointer" 
                />
                <div>
                  <span class="font-bold text-xs text-slate-900 dark:text-slate-100 block">Enable SSH Tunnel (Bastion Host)</span>
                  <span class="text-[11px] text-slate-600 dark:text-slate-400 block">Route database connections securely through an intermediate SSH jump server</span>
                </div>
              </label>
              <span class="px-2 py-0.5 rounded text-[10px] font-bold uppercase tracking-wider bg-indigo-500/20 text-indigo-700 dark:text-indigo-300 border border-indigo-500/30">
                Encrypted Tunnel
              </span>
            </div>

            {#if useSsh}
              <!-- SSH Host & Port -->
              <div class="grid grid-cols-4 gap-3">
                <div class="col-span-3 space-y-1">
                  <label for="ssh-host" class="font-semibold text-slate-700 dark:text-slate-300 text-[11px]">SSH Host / IP Address</label>
                  <input 
                    id="ssh-host" 
                    bind:value={sshHost} 
                    placeholder="bastion.example.com / 139.59.x.x" 
                    class="w-full bg-surface-950 border border-slate-200 dark:border-slate-800 rounded-md px-3 py-1.5 text-slate-900 dark:text-slate-100 text-xs focus:outline-none focus:border-indigo-500 font-mono" 
                  />
                </div>
                <div class="space-y-1">
                  <label for="ssh-port" class="font-semibold text-slate-700 dark:text-slate-300 text-[11px]">SSH Port</label>
                  <input 
                    id="ssh-port" 
                    type="number" 
                    bind:value={sshPort} 
                    class="w-full bg-surface-950 border border-slate-200 dark:border-slate-800 rounded-md px-3 py-1.5 text-slate-900 dark:text-slate-100 text-xs focus:outline-none focus:border-indigo-500" 
                  />
                </div>
              </div>

              <!-- SSH User -->
              <div class="space-y-1">
                <label for="ssh-user" class="font-semibold text-slate-700 dark:text-slate-300 text-[11px]">SSH Username</label>
                <input 
                  id="ssh-user" 
                  bind:value={sshUser} 
                  placeholder="ubuntu / root / ec2-user" 
                  class="w-full bg-surface-950 border border-slate-200 dark:border-slate-800 rounded-md px-3 py-1.5 text-slate-900 dark:text-slate-100 text-xs focus:outline-none focus:border-indigo-500 font-mono" 
                />
              </div>

              <!-- Authentication Method Switcher Tabs -->
              <div class="space-y-2">
                <span class="font-semibold text-slate-700 dark:text-slate-300 text-[11px] block">Authentication Method</span>
                <div class="grid grid-cols-3 gap-2">
                  <button
                    type="button"
                    onclick={() => sshAuthType = 'key'}
                    class="flex items-center justify-center gap-2 py-2 px-3 rounded-lg border text-xs font-semibold transition-all cursor-pointer {sshAuthType === 'key' ? 'bg-indigo-50 dark:bg-indigo-600/20 border-indigo-500 text-indigo-700 dark:text-indigo-300 shadow-xs' : 'bg-surface-950/60 border-slate-200 dark:border-slate-800 text-slate-600 dark:text-slate-400 hover:border-slate-300 dark:hover:border-slate-700 hover:text-slate-900 dark:hover:text-slate-100'}"
                  >
                    <KeyRound size={13} class="text-indigo-500" />
                    <span>Private Key</span>
                  </button>

                  <button
                    type="button"
                    onclick={() => sshAuthType = 'password'}
                    class="flex items-center justify-center gap-2 py-2 px-3 rounded-lg border text-xs font-semibold transition-all cursor-pointer {sshAuthType === 'password' ? 'bg-indigo-50 dark:bg-indigo-600/20 border-indigo-500 text-indigo-700 dark:text-indigo-300 shadow-xs' : 'bg-surface-950/60 border-slate-200 dark:border-slate-800 text-slate-600 dark:text-slate-400 hover:border-slate-300 dark:hover:border-slate-700 hover:text-slate-900 dark:hover:text-slate-100'}"
                  >
                    <Lock size={13} class="text-indigo-500" />
                    <span>Password</span>
                  </button>

                  <button
                    type="button"
                    onclick={() => sshAuthType = 'agent'}
                    class="flex items-center justify-center gap-2 py-2 px-3 rounded-lg border text-xs font-semibold transition-all cursor-pointer {sshAuthType === 'agent' ? 'bg-indigo-50 dark:bg-indigo-600/20 border-indigo-500 text-indigo-700 dark:text-indigo-300 shadow-xs' : 'bg-surface-950/60 border-slate-200 dark:border-slate-800 text-slate-600 dark:text-slate-400 hover:border-slate-300 dark:hover:border-slate-700 hover:text-slate-900 dark:hover:text-slate-100'}"
                  >
                    <Zap size={13} class="text-amber-500" />
                    <span>SSH Agent</span>
                  </button>
                </div>
              </div>

              <!-- Auth Specific Fields -->
              {#if sshAuthType === 'key'}
                <div class="space-y-3 p-3 bg-surface-950/60 border border-slate-200 dark:border-slate-800/80 rounded-xl">
                  <div class="space-y-1">
                    <label for="ssh-key-path" class="font-semibold text-slate-700 dark:text-slate-300 text-[11px]">Private Key File</label>
                    <div class="flex gap-2">
                      <input 
                        id="ssh-key-path" 
                        bind:value={sshKeyPath} 
                        placeholder="~/.ssh/id_rsa or /path/to/key.pem" 
                        class="flex-1 bg-surface-900 border border-slate-200 dark:border-slate-800 rounded-md px-3 py-1.5 text-slate-900 dark:text-slate-100 text-xs font-mono focus:outline-none focus:border-indigo-500" 
                      />
                      <button 
                        type="button" 
                        onclick={handleBrowseSshKey}
                        class="px-3 bg-surface-800 hover:bg-surface-700 text-slate-700 dark:text-slate-300 rounded-md flex items-center gap-1.5 text-xs border border-slate-200 dark:border-slate-700 cursor-pointer"
                      >
                        <FolderOpen size={13} /> Browse
                      </button>
                    </div>
                  </div>

                  <div class="space-y-1">
                    <label for="ssh-passphrase" class="font-semibold text-slate-700 dark:text-slate-300 text-[11px]">Key Passphrase (Optional)</label>
                    <div class="relative">
                      <input 
                        id="ssh-passphrase" 
                        type={showSshPassphrase ? 'text' : 'password'} 
                        bind:value={sshKeyPassphrase} 
                        placeholder="Leave blank if unencrypted" 
                        class="w-full bg-surface-900 border border-slate-200 dark:border-slate-800 rounded-md pl-3 pr-8 py-1.5 text-slate-900 dark:text-slate-100 text-xs focus:outline-none focus:border-indigo-500" 
                      />
                      <button 
                        type="button"
                        onclick={() => showSshPassphrase = !showSshPassphrase}
                        class="absolute right-2.5 top-2 text-slate-400 hover:text-slate-700 dark:hover:text-slate-300 cursor-pointer"
                      >
                        {#if showSshPassphrase}<EyeOff size={13} />{:else}<Eye size={13} />{/if}
                      </button>
                    </div>
                  </div>
                </div>

              {:else if sshAuthType === 'password'}
                <div class="space-y-1 p-3 bg-surface-950/60 border border-slate-200 dark:border-slate-800/80 rounded-xl">
                  <label for="ssh-password" class="font-semibold text-slate-700 dark:text-slate-300 text-[11px]">SSH Password</label>
                  <div class="relative">
                    <input 
                      id="ssh-password" 
                      type={showSshPassword ? 'text' : 'password'} 
                      bind:value={sshPassword} 
                      placeholder="••••••••" 
                      class="w-full bg-surface-900 border border-slate-200 dark:border-slate-800 rounded-md pl-3 pr-8 py-1.5 text-slate-900 dark:text-slate-100 text-xs focus:outline-none focus:border-indigo-500" 
                    />
                    <button 
                      type="button"
                      onclick={() => showSshPassword = !showSshPassword}
                      class="absolute right-2.5 top-2 text-slate-400 hover:text-slate-700 dark:hover:text-slate-300 cursor-pointer"
                    >
                      {#if showSshPassword}<EyeOff size={13} />{:else}<Eye size={13} />{/if}
                    </button>
                  </div>
                </div>

              {:else if sshAuthType === 'agent'}
                <div class="p-3 bg-surface-950/60 border border-slate-200 dark:border-slate-800/80 rounded-xl flex items-start gap-2.5 text-xs text-slate-600 dark:text-slate-400">
                  <Zap size={15} class="shrink-0 text-amber-500 mt-0.5" />
                  <span>FugDB will connect to your active OS SSH Agent (e.g., keys registered via <code>ssh-add</code> or <code>SSH_AUTH_SOCK</code>).</span>
                </div>
              {/if}

              <!-- Test SSH Bastion Button -->
              <div class="pt-2 flex justify-end">
                <button
                  type="button"
                  onclick={handleTestSsh}
                  disabled={isTestingSsh}
                  class="px-3 py-1.5 text-xs font-semibold rounded-lg bg-surface-800 hover:bg-surface-700 text-slate-800 dark:text-slate-200 border border-slate-200 dark:border-slate-700 flex items-center gap-1.5 cursor-pointer disabled:opacity-50 transition-colors"
                >
                  <Server size={13} class={isTestingSsh ? 'animate-spin text-indigo-500' : 'text-indigo-400'} />
                  <span>{isTestingSsh ? 'Testing SSH Bastion...' : 'Test SSH Tunnel'}</span>
                </button>
              </div>
            {/if}
          </div>

        {:else if activeTab === 'ssl'}
          <!-- SSL Tab -->
          <div class="space-y-3">
            <span class="font-semibold text-slate-600 dark:text-slate-400 text-[10px] uppercase block">SSL Mode</span>
            <div class="space-y-2">
              {#each [
                { id: 'disable', label: 'Disable', desc: 'No SSL encryption (local dev only)' },
                { id: 'prefer', label: 'Prefer', desc: 'Try SSL if supported, fallback to plaintext' },
                { id: 'require', label: 'Require', desc: 'Force SSL connection without certificate validation' }
              ] as opt}
                <label class="flex items-start gap-2.5 p-3 rounded-lg border border-slate-200 dark:border-slate-800/80 bg-surface-950/40 cursor-pointer hover:border-slate-300 dark:hover:border-slate-700">
                  <input type="radio" name="ssl" value={opt.id} bind:group={sslMode} class="mt-0.5 text-indigo-600" />
                  <div>
                    <span class="font-semibold text-slate-800 dark:text-slate-200 block">{opt.label}</span>
                    <span class="text-[11px] text-slate-500">{opt.desc}</span>
                  </div>
                </label>
              {/each}
            </div>
          </div>
        {/if}
      </div>

      <!-- Footer Actions -->
      <div class="px-6 py-3 border-t border-slate-200 dark:border-slate-800 bg-surface-950/50 flex items-center justify-between">
        <button 
          type="button"
          onclick={handleTest}
          disabled={isTesting}
          class="px-3.5 py-1.5 text-xs font-semibold text-slate-700 dark:text-slate-300 bg-surface-800 hover:bg-surface-700 hover:text-slate-950 dark:hover:text-white rounded-md border border-slate-200 dark:border-slate-700 transition-colors disabled:opacity-50"
        >
          {isTesting ? 'Testing...' : 'Test Connection'}
        </button>

        <div class="flex items-center gap-2">
          <button type="button" onclick={onClose} class="px-3 py-1.5 text-xs text-slate-500 dark:text-slate-400 hover:text-slate-800 dark:hover:text-slate-200">
            Cancel
          </button>
          <button 
            type="button"
            onclick={handleSaveAndConnect}
            disabled={isSaving}
            class="px-4 py-1.5 text-xs font-semibold rounded-md bg-indigo-600 hover:bg-indigo-500 text-white disabled:bg-slate-300 dark:disabled:bg-slate-800 transition-all shadow-md shadow-indigo-600/20"
          >
            {isSaving ? 'Connecting...' : 'Save & Connect'}
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
