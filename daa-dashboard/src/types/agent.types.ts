// Agent Types
export interface Agent {
  id: string;
  name: string;
  type: AgentType;
  status: AgentStatus;
  version: string;
  description?: string;
  customer: {
    id: string;
    name: string;
  };
  environment: 'production' | 'staging' | 'development';
  createdAt: string;
  updatedAt: string;
  lastActivity?: string;
  metrics: AgentMetrics;
  configuration: AgentConfiguration;
  capabilities: string[];
}

export type AgentType = 
  | 'treasury'
  | 'defi'
  | 'security'
  | 'research'
  | 'governance'
  | 'custom';

export type AgentStatus = 
  | 'active'
  | 'idle'
  | 'training'
  | 'error'
  | 'maintenance'
  | 'stopped';

export interface AgentMetrics {
  uptime: number;
  healthScore: number;
  responseTime: number;
  successRate: number;
  errorRate: number;
  totalTransactions: number;
  totalEarnings: number;
  totalSpending: number;
  resourceUtilization: {
    cpu: number;
    memory: number;
    storage: number;
    network: number;
  };
}

export interface AgentConfiguration {
  aiModel: string;
  maxDailySpend: number;
  riskTolerance: number;
  operatingHours?: string;
  geographicRestrictions?: string[];
  rules: AgentRule[];
  resources: ResourceAllocation;
  monitoring: MonitoringConfig;
}

export interface AgentRule {
  id: string;
  type: 'governance' | 'operational' | 'security' | 'economic';
  name: string;
  description: string;
  enabled: boolean;
  conditions: RuleCondition[];
  actions: RuleAction[];
}

export interface RuleCondition {
  field: string;
  operator: 'equals' | 'not_equals' | 'greater_than' | 'less_than' | 'contains' | 'in';
  value: any;
}

export interface RuleAction {
  type: string;
  parameters: Record<string, any>;
}

export interface ResourceAllocation {
  computeUnits: number;
  storageGB: number;
  networkBandwidthMbps: number;
  priority: 'low' | 'medium' | 'high' | 'critical';
}

export interface MonitoringConfig {
  alertThresholds: AlertThreshold[];
  reportingFrequency: string;
  auditLevel: 'basic' | 'detailed' | 'comprehensive';
  metricsRetention: number;
}

export interface AlertThreshold {
  metric: string;
  threshold: number;
  comparison: 'above' | 'below';
  severity: 'info' | 'warning' | 'error' | 'critical';
}

// Agent Creation/Update
export interface AgentConfig {
  name: string;
  type: AgentType;
  description?: string;
  customerId: string;
  environment: 'production' | 'staging' | 'development';
  configuration: Partial<AgentConfiguration>;
}

// Agent Filters
export interface AgentFilters {
  status?: AgentStatus | 'all';
  type?: AgentType | 'all';
  customer?: string | 'all';
  environment?: 'production' | 'staging' | 'development' | 'all';
  search?: string;
}

// Agent Actions
export interface AgentAction {
  id: string;
  agentId: string;
  type: 'start' | 'stop' | 'restart' | 'update' | 'delete';
  status: 'pending' | 'in_progress' | 'completed' | 'failed';
  createdAt: string;
  completedAt?: string;
  error?: string;
}

// Swarm Types
export interface Swarm {
  id: string;
  name: string;
  objective: string;
  strategy: 'collective-intelligence' | 'competitive' | 'collaborative';
  status: 'active' | 'paused' | 'completed' | 'failed';
  participants: SwarmParticipant[];
  constraints: SwarmConstraints;
  metrics: SwarmMetrics;
  createdAt: string;
  updatedAt: string;
}

export interface SwarmParticipant {
  agentId: string;
  role: string;
  weight: number;
  status: 'active' | 'inactive';
  contribution: number;
}

export interface SwarmConstraints {
  maxDuration: number;
  budgetLimit: number;
  riskThreshold: number;
  complianceRequirements: string[];
}

export interface SwarmMetrics {
  progress: number;
  totalCost: number;
  efficiency: number;
  consensusLevel: number;
  objectives: {
    total: number;
    completed: number;
  };
}