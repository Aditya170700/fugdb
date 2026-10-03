import { api } from '$lib/api/client';
import type { AiProvider, AiProviderConfig, AiSqlResponse, OllamaModelInfo } from '$lib/api/types';
import { connectionStore } from '$lib/state/connection.svelte';
import { tabsStore } from '$lib/state/tabs.svelte';

const STORAGE_KEY = 'fugdb_ai_config_v2';

function loadInitialConfig() {
  const defaultApiKeys: Record<string, string> = {
    ollama: '',
    openai: '',
    gemini: '',
    deepseek: '',
    anthropic: '',
    custom: '',
  };
  const defaultModels: Record<string, string> = {
    ollama: 'qwen2.5-coder:latest',
    openai: 'gpt-4o-mini',
    gemini: 'gemini-1.5-flash',
    deepseek: 'deepseek-coder',
    anthropic: 'claude-3-5-sonnet-20241022',
    custom: 'custom-model',
  };
  const defaultEndpoints: Record<string, string> = {
    ollama: 'http://localhost:11434',
    openai: 'https://api.openai.com/v1',
    gemini: 'https://generativelanguage.googleapis.com/v1beta',
    deepseek: 'https://api.deepseek.com/v1',
    anthropic: 'https://api.anthropic.com/v1',
    custom: 'http://localhost:8000/v1',
  };

  let provider: AiProvider = 'ollama';
  let apiKeys = { ...defaultApiKeys };
  let models = { ...defaultModels };
  let endpoints = { ...defaultEndpoints };
  let temperature = 0.2;
  let promptHistory: Array<{ prompt: string; sql: string; timestamp: number }> = [];

  if (typeof window !== 'undefined') {
    try {
      const savedV2 = localStorage.getItem(STORAGE_KEY);
      if (savedV2) {
        const parsed = JSON.parse(savedV2);
        if (parsed.provider) provider = parsed.provider;
        if (parsed.providerApiKeys) apiKeys = { ...defaultApiKeys, ...parsed.providerApiKeys };
        if (parsed.providerModels) models = { ...defaultModels, ...parsed.providerModels };
        if (parsed.providerEndpoints) endpoints = { ...defaultEndpoints, ...parsed.providerEndpoints };
        if (parsed.temperature !== undefined) temperature = parsed.temperature;
        if (parsed.promptHistory) promptHistory = parsed.promptHistory;
      } else {
        // Fallback migration from legacy storage
        const legacy = localStorage.getItem('fugdb_ai_config');
        if (legacy) {
          const parsed = JSON.parse(legacy);
          if (parsed.provider) provider = parsed.provider;
          if (parsed.apiKey) apiKeys[provider] = parsed.apiKey;
          if (parsed.model) models[provider] = parsed.model;
          if (parsed.endpoint) endpoints[provider] = parsed.endpoint;
          if (parsed.temperature !== undefined) temperature = parsed.temperature;
          if (parsed.promptHistory) promptHistory = parsed.promptHistory;
        }
      }
    } catch (e) {
      console.error('Failed to load AI settings from localStorage:', e);
    }
  }

  return { provider, apiKeys, models, endpoints, temperature, promptHistory };
}

function createAiStore() {
  const initial = loadInitialConfig();

  let isDrawerOpen = $state<boolean>(false);
  let isSettingsOpen = $state<boolean>(false);
  let isFixModalOpen = $state<boolean>(false);

  // Configuration - Isolated per provider so keys and models don't mix
  let provider = $state<AiProvider>(initial.provider);
  let providerApiKeys = $state<Record<string, string>>(initial.apiKeys);
  let providerModels = $state<Record<string, string>>(initial.models);
  let providerEndpoints = $state<Record<string, string>>(initial.endpoints);
  let temperature = $state<number>(initial.temperature);

  // Status & Runtime
  let isGenerating = $state<boolean>(false);
  let isTesting = $state<boolean>(false);
  let testResult = $state<{ success: boolean; message: string } | null>(null);
  let ollamaModels = $state<OllamaModelInfo[]>([]);
  let isLoadingOllama = $state<boolean>(false);
  let lastResponse = $state<AiSqlResponse | null>(null);
  let errorMessage = $state<string | null>(null);

  // Fix with AI State
  let fixTarget = $state<{
    tabId: string;
    failedSql: string;
    errorMessage: string;
    fixedResponse: AiSqlResponse | null;
    isFixing: boolean;
    fixError: string | null;
  } | null>(null);

  // History
  let promptHistory = $state<Array<{ prompt: string; sql: string; timestamp: number }>>(initial.promptHistory);

  function saveConfig() {
    if (typeof window === 'undefined') return;
    try {
      const config = {
        provider,
        providerApiKeys,
        providerModels,
        providerEndpoints,
        temperature,
        promptHistory: promptHistory.slice(-20) // Keep last 20 queries
      };
      localStorage.setItem(STORAGE_KEY, JSON.stringify(config));
    } catch (e) {
      console.error('Failed to save AI settings to localStorage:', e);
    }
  }

  const currentConfig = $derived<AiProviderConfig>({
    provider,
    model: providerModels[provider] || 'default-model',
    apiKey: providerApiKeys[provider] ? providerApiKeys[provider] : undefined,
    endpoint: providerEndpoints[provider] ? providerEndpoints[provider] : undefined,
    temperature
  });

  async function loadOllamaModels() {
    if (provider !== 'ollama') return;
    isLoadingOllama = true;
    try {
      const ep = providerEndpoints['ollama'] || 'http://localhost:11434';
      const models = await api.listOllamaModels(ep);
      ollamaModels = models;
      if (models.length > 0 && !models.some(m => m.name === providerModels['ollama'])) {
        providerModels['ollama'] = models[0].name;
        saveConfig();
      }
    } catch (e) {
      console.warn('Could not load Ollama models:', e);
    } finally {
      isLoadingOllama = false;
    }
  }

  async function testConnection(): Promise<boolean> {
    isTesting = true;
    testResult = null;
    try {
      const msg = await api.testAiConnection(currentConfig);
      testResult = { success: true, message: msg };
      return true;
    } catch (e: any) {
      testResult = {
        success: false,
        message: typeof e === 'string' ? e : e?.message || String(e)
      };
      return false;
    } finally {
      isTesting = false;
    }
  }

  async function generateSql(userPrompt: string, selectedTables?: string[]): Promise<AiSqlResponse | null> {
    const activeConn = connectionStore.activeConnection;
    if (!activeConn) {
      errorMessage = 'No active database connection. Please connect to a database first.';
      return null;
    }

    const schema = connectionStore.schemas[activeConn.id];
    if (!schema || schema.tables.length === 0) {
      errorMessage = 'No schema metadata found. Please refresh the database schema first.';
      return null;
    }

    isGenerating = true;
    errorMessage = null;
    try {
      const resp = await api.generateSqlFromPrompt(
        activeConn.id,
        currentConfig,
        userPrompt,
        selectedTables,
        activeConn.driver
      );
      lastResponse = resp;
      promptHistory = [
        { prompt: userPrompt, sql: resp.sql, timestamp: Date.now() },
        ...promptHistory.filter(h => h.prompt !== userPrompt).slice(0, 19)
      ];
      saveConfig();
      return resp;
    } catch (e: any) {
      errorMessage = typeof e === 'string' ? e : e?.message || String(e);
      return null;
    } finally {
      isGenerating = false;
    }
  }

  async function fixSql(failedSql: string, rawError: string): Promise<AiSqlResponse | null> {
    const activeConn = connectionStore.activeConnection;
    if (!activeConn) return null;

    isGenerating = true;
    errorMessage = null;
    try {
      const fixed = await api.fixSqlError(
        activeConn.id,
        currentConfig,
        failedSql,
        rawError,
        activeConn.driver
      );
      return fixed;
    } catch (e: any) {
      errorMessage = typeof e === 'string' ? e : e?.message || String(e);
      return null;
    } finally {
      isGenerating = false;
    }
  }

  async function triggerFix(tabId: string, failedSql: string, rawError: string) {
    const activeConn = connectionStore.activeConnection;
    if (!activeConn) {
      errorMessage = 'No active database connection. Please connect to a database first.';
      return;
    }

    isFixModalOpen = true;
    fixTarget = {
      tabId,
      failedSql,
      errorMessage: rawError,
      fixedResponse: null,
      isFixing: true,
      fixError: null
    };

    try {
      const fixed = await api.fixSqlError(
        activeConn.id,
        currentConfig,
        failedSql,
        rawError,
        activeConn.driver
      );
      if (fixTarget && fixTarget.tabId === tabId) {
        fixTarget.fixedResponse = fixed;
        fixTarget.isFixing = false;
      }
    } catch (e: any) {
      if (fixTarget && fixTarget.tabId === tabId) {
        fixTarget.fixError = typeof e === 'string' ? e : e?.message || String(e);
        fixTarget.isFixing = false;
      }
    }
  }

  async function retryFix() {
    if (!fixTarget) return;
    await triggerFix(fixTarget.tabId, fixTarget.failedSql, fixTarget.errorMessage);
  }

  function applyFix(tabId: string, runImmediately: boolean = false) {
    if (!fixTarget?.fixedResponse?.sql) return;
    const fixedSql = fixTarget.fixedResponse.sql;
    tabsStore.updateTabSql(tabId, fixedSql);
    if (runImmediately) {
      tabsStore.runTabQuery(tabId);
    }
    isFixModalOpen = false;
    fixTarget = null;
  }

  function closeFixModal() {
    isFixModalOpen = false;
    fixTarget = null;
  }

  function setProvider(newProvider: AiProvider) {
    provider = newProvider;
    testResult = null;
    if (newProvider === 'ollama') {
      loadOllamaModels();
    }
    saveConfig();
  }

  return {
    get isDrawerOpen() { return isDrawerOpen; },
    set isDrawerOpen(v) { isDrawerOpen = v; },
    get isSettingsOpen() { return isSettingsOpen; },
    set isSettingsOpen(v) { isSettingsOpen = v; },
    get isFixModalOpen() { return isFixModalOpen; },
    set isFixModalOpen(v) { isFixModalOpen = v; },
    get fixTarget() { return fixTarget; },

    get provider() { return provider; },
    set provider(v) { setProvider(v); },

    get model() { return providerModels[provider] || ''; },
    set model(v) { providerModels[provider] = v; saveConfig(); },

    get apiKey() { return providerApiKeys[provider] || ''; },
    set apiKey(v) { 
      providerApiKeys[provider] = v; 
      saveConfig(); 
      if (v && v.trim() !== '') {
        api.saveKeyringCredential(`ai_key_${provider}`, v).catch(() => {});
      } else {
        api.deleteKeyringCredential(`ai_key_${provider}`).catch(() => {});
      }
    },

    get endpoint() { return providerEndpoints[provider] || ''; },
    set endpoint(v) { providerEndpoints[provider] = v; saveConfig(); },

    get temperature() { return temperature; },
    set temperature(v) { temperature = v; saveConfig(); },

    get currentConfig() { return currentConfig; },
    get isGenerating() { return isGenerating; },
    get isTesting() { return isTesting; },
    get testResult() { return testResult; },
    set testResult(v) { testResult = v; },
    get ollamaModels() { return ollamaModels; },
    get isLoadingOllama() { return isLoadingOllama; },
    get lastResponse() { return lastResponse; },
    set lastResponse(v) { lastResponse = v; },
    get errorMessage() { return errorMessage; },
    set errorMessage(v) { errorMessage = v; },
    get promptHistory() { return promptHistory; },

    loadOllamaModels,
    testConnection,
    generateSql,
    fixSql,
    triggerFix,
    retryFix,
    applyFix,
    closeFixModal,
    saveConfig,
    openDrawer: () => { isDrawerOpen = true; },
    closeDrawer: () => { isDrawerOpen = false; },
    openSettings: () => { isSettingsOpen = true; },
    closeSettings: () => { isSettingsOpen = false; }
  };
}

export const aiStore = createAiStore();
