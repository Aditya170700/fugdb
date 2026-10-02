import type { Environment } from '$lib/api/types';
import type { SqlRiskAssessment } from '$lib/utils/safetyGuard';

export interface SafetyRequestOptions {
  connectionId: string;
  connectionName: string;
  databaseName: string;
  environment: Environment;
  sql: string;
  risk: SqlRiskAssessment;
}

export class SafetyStore {
  isOpen = $state(false);
  connectionId = $state('');
  connectionName = $state('');
  databaseName = $state('');
  environment = $state<Environment>('production');
  sql = $state('');
  risk = $state<SqlRiskAssessment | null>(null);

  private resolver: ((proceed: boolean) => void) | null = null;

  async requestConfirmation(options: SafetyRequestOptions): Promise<boolean> {
    this.connectionId = options.connectionId;
    this.connectionName = options.connectionName;
    this.databaseName = options.databaseName;
    this.environment = options.environment;
    this.sql = options.sql;
    this.risk = options.risk;
    this.isOpen = true;

    return new Promise<boolean>((resolve) => {
      this.resolver = resolve;
    });
  }

  confirm() {
    this.isOpen = false;
    if (this.resolver) {
      this.resolver(true);
      this.resolver = null;
    }
  }

  cancel() {
    this.isOpen = false;
    if (this.resolver) {
      this.resolver(false);
      this.resolver = null;
    }
  }
}

export const safetyStore = new SafetyStore();
