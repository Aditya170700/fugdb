import { api } from '$lib/api/client';
import type { AiProvider, AiProviderConfig, AiSqlResponse, OllamaModelInfo } from '$lib/api/types';
import { connectionStore } from '$lib/state/connection.svelte';

const STORAGE_KEY = 'fugdb_ai_config';

function createAiStore() {
  let isDrawerOpen = $state<boolean>(false);
  let isSettingsOpen = $state<boolean>(false);

  // Configuration
  let provider = $state<AiProvider>('ollama');
  let model = $state<string>('qwen2.5-coder:latest');
  let apiKey = $state<string>('');
  let endpoint = $state<string>('http://localhost:11434');
  let temperature = $state<number>(0.2);

  // Status & Runtime
  let isGenerating = $state<boolean>(false);
  let isTesting = $state<boolean>(false);
  let testResult = $state<{ success: boolean; message: string } | null>(null);
  let ollamaModels = $state<OllamaModelInfo[]>([]);
  let isLoadingOllama = $state<boolean>(false);
  let lastResponse = $state<AiSqlResponse | null>(null);
  let errorMessage = $state<string | null>(null);

  // History
  let promptHistory = $state<Array<{ prompt: string; sql: string; timestamp: number }>>([]);

  // Load from localStorage on initialization
  if (typeof window !== 'undefined') {
    try {
      const saved = localStorage.getItem(STORAGE_KEY);
      if (saved) {
        const parsed = JSON.parse(saved);
        if (parsed.provider) provider = parsed.provider;
        if (parsed.model) model = parsed.model;
        if (parsed.apiKey) apiKey = parsed.apiKey;
        if (parsed.endpoint) endpoint = parsed.endpoint;
        if (parsed.temperature !== undefined) temperature = parsed.temperature;
        if (parsed.promptHistory) promptHistory = parsed.promptHistory;
      }
    } catch (e) {
      console.error('Failed to load AI settings from localStorage:', e);
    }
  }

  function saveConfig() {
    if (typeof window === 'undefined') return;
    try {
      const config = {
        provider,
        model,
        apiKey,
        endpoint,
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
    model,
    apiKey: apiKey ? apiKey : undefined,
    endpoint: endpoint ? endpoint : undefined,
    temperature
  });

  async function loadOllamaModels() {
    if (provider !== 'ollama') return;
    isLoadingOllama = true;
    try {
      const models = await api.listOllamaModels(endpoint);
      ollamaModels = models;
      if (models.length > 0 && !models.some(m => m.name === model)) {
        model = models[0].name;
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
    const connId = connectionStore.activeConnection?.id;
    if (!connId) {
      errorMessage = 'Please connect to a database first.';
      return null;
    }
    if (!userPrompt.trim()) return null;

    isGenerating = true;
    errorMessage = null;

    try {
      const response = await api.generateSqlFromPrompt(
        connId,
        currentConfig,
        userPrompt,
        selectedTables
      );
      lastResponse = response;
      promptHistory = [
        { prompt: userPrompt, sql: response.sql, timestamp: Date.now() },
        ...promptHistory.filter(h => h.prompt !== userPrompt).slice(0, 19)
      ];
      saveConfig();
      return response;
    } catch (e: any) {
      errorMessage = typeof e === 'string' ? e : e?.message || String(e);
      return null;
    } finally {
      isGenerating = false;
    }
  }

  async function fixSql(sql: string, queryError: string): Promise<AiSqlResponse | null> {
    const connId = connectionStore.activeConnection?.id;
    if (!connId) {
      errorMessage = 'Please connect to a database first.';
      return null;
    }

    isGenerating = true;
    errorMessage = null;

    try {
      const response = await api.fixSqlError(
        connId,
        currentConfig,
        sql,
        queryError
      );
      lastResponse = response;
      return response;
    } catch (e: any) {
      errorMessage = typeof e === 'string' ? e : e?.message || String(e);
      return null;
    } finally {
      isGenerating = false;
    }
  }

  function setProvider(newProvider: AiProvider) {
    provider = newProvider;
    switch (newProvider) {
      case 'ollama':
        model = 'qwen2.5-coder:latest';
        endpoint = 'http://localhost:11434';
        loadOllamaModels();
        break;
      case 'openai':
        model = 'gpt-4o-mini';
        endpoint = 'https://api.openai.com/v1';
        break;
      case 'gemini':
        model = 'gemini-1.5-flash';
        endpoint = 'https://generativelanguage.googleapis.com/v1beta';
        break;
      case 'anthropic':
        model = 'claude-3-5-sonnet-20241022';
        endpoint = 'https://api.anthropic.com/v1';
        break;
      case 'deepseek':
        model = 'deepseek-coder';
        endpoint = 'https://api.deepseek.com/v1';
        break;
      case 'custom':
        model = 'custom-model';
        endpoint = 'http://localhost:8000/v1';
        break;
    }
    saveConfig();
  }

  return {
    get isDrawerOpen() { return isDrawerOpen; },
    set isDrawerOpen(v) { isDrawerOpen = v; },
    get isSettingsOpen() { return isSettingsOpen; },
    set isSettingsOpen(v) { isSettingsOpen = v; },

    get provider() { return provider; },
    set provider(v) { setProvider(v); },
    get model() { return model; },
    set model(v) { model = v; saveConfig(); },
    get apiKey() { return apiKey; },
    set apiKey(v) { apiKey = v; saveConfig(); },
    get endpoint() { return endpoint; },
    set endpoint(v) { endpoint = v; saveConfig(); },
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
    saveConfig,
    openDrawer: () => { isDrawerOpen = true; },
    closeDrawer: () => { isDrawerOpen = false; },
    openSettings: () => { isSettingsOpen = true; },
    closeSettings: () => { isSettingsOpen = false; }
  };
}

export const aiStore = createAiStore();
