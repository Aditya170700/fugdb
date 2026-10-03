import { api } from '../api/client';
import type {
  RedisKeyItem,
  RedisKeyDetail,
  RedisServerInfo,
  RedisCliResponse,
  RedisKeyType
} from '../api/types';

export interface RedisConnectionState {
  connectionId: string;
  selectedDb: number;
  searchPattern: string;
  keys: RedisKeyItem[];
  totalKeys: number;
  isLoadingKeys: boolean;
  selectedKeyName: string | null;
  selectedKeyDetail: RedisKeyDetail | null;
  isLoadingDetail: boolean;
  serverInfo: RedisServerInfo | null;
  cliLogs: RedisCliResponse[];
  activeSubView: 'browser' | 'cli' | 'info';
}

export class RedisStore {
  states = $state<Record<string, RedisConnectionState>>({});

  ensureState(connectionId: string): RedisConnectionState {
    if (!this.states[connectionId]) {
      this.states[connectionId] = {
        connectionId,
        selectedDb: 0,
        searchPattern: '*',
        keys: [],
        totalKeys: 0,
        isLoadingKeys: false,
        selectedKeyName: null,
        selectedKeyDetail: null,
        isLoadingDetail: false,
        serverInfo: null,
        cliLogs: [
          {
            command: 'INFO SERVER',
            response: '# Server ready. Type redis commands below (e.g. GET key, KEYS *, INFO)',
            responseType: 'status',
            durationMs: 0
          }
        ],
        activeSubView: 'browser'
      };
    }
    return this.states[connectionId];
  }

  getState(connectionId: string): RedisConnectionState {
    if (this.states[connectionId]) {
      return this.states[connectionId];
    }
    return {
      connectionId,
      selectedDb: 0,
      searchPattern: '*',
      keys: [],
      totalKeys: 0,
      isLoadingKeys: false,
      selectedKeyName: null,
      selectedKeyDetail: null,
      isLoadingDetail: false,
      serverInfo: null,
      cliLogs: [
        {
          command: 'INFO SERVER',
          response: '# Server ready. Type redis commands below (e.g. GET key, KEYS *, INFO)',
          responseType: 'status',
          durationMs: 0
        }
      ],
      activeSubView: 'browser'
    };
  }

  async loadKeys(connectionId: string, pattern?: string, dbIndex?: number) {
    const s = this.ensureState(connectionId);
    s.isLoadingKeys = true;
    if (dbIndex !== undefined) s.selectedDb = dbIndex;
    if (pattern !== undefined) s.searchPattern = pattern;

    try {
      const res = await api.scanRedisKeys(connectionId, s.searchPattern, s.selectedDb, 0, 300);
      s.keys = res.keys;
      s.totalKeys = res.totalKeys;

      // If previously selected key still exists, keep it
      if (s.selectedKeyName && !s.keys.some(k => k.key === s.selectedKeyName)) {
        s.selectedKeyName = null;
        s.selectedKeyDetail = null;
      }
    } catch (err) {
      console.error('Failed to scan Redis keys:', err);
    } finally {
      s.isLoadingKeys = false;
    }
  }

  async selectKey(connectionId: string, keyName: string) {
    const s = this.ensureState(connectionId);
    s.selectedKeyName = keyName;
    s.isLoadingDetail = true;

    try {
      const detail = await api.getRedisKeyDetail(connectionId, keyName, s.selectedDb);
      s.selectedKeyDetail = detail;
    } catch (err) {
      console.error('Failed to get Redis key detail:', err);
      s.selectedKeyDetail = null;
    } finally {
      s.isLoadingDetail = false;
    }
  }

  async refreshSelectedKey(connectionId: string) {
    const s = this.ensureState(connectionId);
    if (!s.selectedKeyName) return;
    await this.selectKey(connectionId, s.selectedKeyName);
    // Also update the key item in the list if TTL/size changed
    try {
      const res = await api.scanRedisKeys(connectionId, s.selectedKeyName, s.selectedDb, 0, 10);
      const match = res.keys.find(k => k.key === s.selectedKeyName);
      if (match) {
        const idx = s.keys.findIndex(k => k.key === s.selectedKeyName);
        if (idx !== -1) {
          s.keys[idx] = match;
        }
      }
    } catch {}
  }

  async updateTtl(connectionId: string, key: string, ttl: number) {
    const s = this.ensureState(connectionId);
    try {
      await api.setRedisKeyTtl(connectionId, key, ttl, s.selectedDb);
      await this.refreshSelectedKey(connectionId);
    } catch (err) {
      console.error('Failed to update TTL:', err);
      throw err;
    }
  }

  async updateStringValue(connectionId: string, key: string, value: string, ttl?: number) {
    const s = this.ensureState(connectionId);
    try {
      await api.setRedisString(connectionId, key, value, ttl, s.selectedDb);
      await this.refreshSelectedKey(connectionId);
    } catch (err) {
      console.error('Failed to set string:', err);
      throw err;
    }
  }

  async setHashField(connectionId: string, key: string, field: string, value: string) {
    const s = this.ensureState(connectionId);
    try {
      await api.setRedisHashField(connectionId, key, field, value, s.selectedDb);
      await this.refreshSelectedKey(connectionId);
    } catch (err) {
      console.error('Failed to set hash field:', err);
      throw err;
    }
  }

  async deleteHashField(connectionId: string, key: string, field: string) {
    const s = this.ensureState(connectionId);
    try {
      await api.deleteRedisHashField(connectionId, key, field, s.selectedDb);
      await this.refreshSelectedKey(connectionId);
    } catch (err) {
      console.error('Failed to delete hash field:', err);
      throw err;
    }
  }

  async pushListElement(connectionId: string, key: string, value: string, position: 'left' | 'right') {
    const s = this.ensureState(connectionId);
    try {
      await api.pushRedisListElement(connectionId, key, value, position, s.selectedDb);
      await this.refreshSelectedKey(connectionId);
    } catch (err) {
      console.error('Failed to push list element:', err);
      throw err;
    }
  }

  async removeListElement(connectionId: string, key: string, value: string) {
    const s = this.ensureState(connectionId);
    try {
      await api.removeRedisListElement(connectionId, key, value, s.selectedDb);
      await this.refreshSelectedKey(connectionId);
    } catch (err) {
      console.error('Failed to remove list element:', err);
      throw err;
    }
  }

  async addSetMember(connectionId: string, key: string, member: string) {
    const s = this.ensureState(connectionId);
    try {
      await api.addRedisSetMember(connectionId, key, member, s.selectedDb);
      await this.refreshSelectedKey(connectionId);
    } catch (err) {
      console.error('Failed to add set member:', err);
      throw err;
    }
  }

  async removeSetMember(connectionId: string, key: string, member: string) {
    const s = this.ensureState(connectionId);
    try {
      await api.removeRedisSetMember(connectionId, key, member, s.selectedDb);
      await this.refreshSelectedKey(connectionId);
    } catch (err) {
      console.error('Failed to remove set member:', err);
      throw err;
    }
  }

  async addZSetMember(connectionId: string, key: string, member: string, score: number) {
    const s = this.ensureState(connectionId);
    try {
      await api.addRedisZSetMember(connectionId, key, member, score, s.selectedDb);
      await this.refreshSelectedKey(connectionId);
    } catch (err) {
      console.error('Failed to add zset member:', err);
      throw err;
    }
  }

  async removeZSetMember(connectionId: string, key: string, member: string) {
    const s = this.ensureState(connectionId);
    try {
      await api.removeRedisZSetMember(connectionId, key, member, s.selectedDb);
      await this.refreshSelectedKey(connectionId);
    } catch (err) {
      console.error('Failed to remove zset member:', err);
      throw err;
    }
  }

  async deleteKey(connectionId: string, key: string) {
    const s = this.ensureState(connectionId);
    try {
      await api.deleteRedisKeys(connectionId, [key], s.selectedDb);
      if (s.selectedKeyName === key) {
        s.selectedKeyName = null;
        s.selectedKeyDetail = null;
      }
      await this.loadKeys(connectionId);
    } catch (err) {
      console.error('Failed to delete key:', err);
      throw err;
    }
  }

  async renameKey(connectionId: string, oldKey: string, newKey: string) {
    const s = this.ensureState(connectionId);
    try {
      await api.renameRedisKey(connectionId, oldKey, newKey, s.selectedDb);
      s.selectedKeyName = newKey;
      await this.loadKeys(connectionId);
      await this.selectKey(connectionId, newKey);
    } catch (err) {
      console.error('Failed to rename key:', err);
      throw err;
    }
  }

  async createNewKey(
    connectionId: string,
    params: {
      key: string;
      keyType: RedisKeyType;
      value: string;
      ttl?: number;
      field?: string;
      score?: number;
    }
  ) {
    const s = this.ensureState(connectionId);
    const { key, keyType, value, ttl, field, score } = params;

    try {
      if (keyType === 'string') {
        await api.setRedisString(connectionId, key, value, ttl, s.selectedDb);
      } else if (keyType === 'hash') {
        await api.setRedisHashField(connectionId, key, field || 'field1', value || 'value1', s.selectedDb);
      } else if (keyType === 'list') {
        await api.pushRedisListElement(connectionId, key, value || 'item1', 'right', s.selectedDb);
      } else if (keyType === 'set') {
        await api.addRedisSetMember(connectionId, key, value || 'member1', s.selectedDb);
      } else if (keyType === 'zset') {
        await api.addRedisZSetMember(connectionId, key, value || 'member1', score ?? 1.0, s.selectedDb);
      }

      if (ttl && ttl > 0 && keyType !== 'string') {
        await api.setRedisKeyTtl(connectionId, key, ttl, s.selectedDb);
      }

      await this.loadKeys(connectionId);
      await this.selectKey(connectionId, key);
    } catch (err) {
      console.error('Failed to create new key:', err);
      throw err;
    }
  }

  async executeCli(connectionId: string, commandLine: string) {
    const s = this.ensureState(connectionId);
    if (!commandLine.trim()) return;

    try {
      const res = await api.executeRedisCliCommand(connectionId, commandLine, s.selectedDb);
      s.cliLogs = [...s.cliLogs, res];
    } catch (err: any) {
      const errMsg = typeof err === 'string' ? err : err?.message || JSON.stringify(err);
      s.cliLogs = [
        ...s.cliLogs,
        {
          command: commandLine,
          response: `(error) ${errMsg}`,
          responseType: 'error',
          durationMs: 0
        }
      ];
    }
  }

  async loadServerInfo(connectionId: string) {
    const s = this.ensureState(connectionId);
    try {
      const info = await api.getRedisServerInfo(connectionId);
      s.serverInfo = info;
    } catch (err) {
      console.error('Failed to get Redis server info:', err);
    }
  }

  async flushCurrentDb(connectionId: string) {
    const s = this.ensureState(connectionId);
    try {
      await api.flushRedisDb(connectionId, s.selectedDb);
      s.selectedKeyName = null;
      s.selectedKeyDetail = null;
      await this.loadKeys(connectionId);
    } catch (err) {
      console.error('Failed to flush Redis DB:', err);
      throw err;
    }
  }
}

export const redisStore = new RedisStore();
