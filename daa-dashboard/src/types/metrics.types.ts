// Metrics Types
export interface GlobalMetrics {
  totalAgents: MetricValue;
  activeCustomers: MetricValue;
  monthlyRevenue: MetricValue;
  systemUptime: MetricValue;
  networkStatus: MetricValue;
  securityStatus: MetricValue;
}

export interface MetricValue {
  value: string | number;
  trend?: number;
  status?: 'healthy' | 'warning' | 'critical';
  change?: number;
  unit?: string;
}

export interface SystemStatus {
  status: 'operational' | 'degraded' | 'outage';
  uptime: number;
  lastIncident?: Date;
  services: ServiceStatus[];
}

export interface ServiceStatus {
  name: string;
  status: 'operational' | 'degraded' | 'outage';
  latency: number;
  errorRate: number;
  lastCheck: Date;
}

export interface NetworkStatus {
  connectedNodes: number;
  totalNodes: number;
  networkHealth: number;
  avgLatency: number;
  bandwidth: {
    in: number;
    out: number;
  };
  regions: RegionStatus[];
}

export interface RegionStatus {
  name: string;
  nodes: number;
  status: 'online' | 'degraded' | 'offline';
  latency: number;
}

export interface PerformanceMetrics {
  timestamp: string;
  cpu: number;
  memory: number;
  disk: number;
  network: {
    in: number;
    out: number;
  };
  requestsPerSecond: number;
  avgResponseTime: number;
  errorRate: number;
}

export interface ChartData {
  labels: string[];
  datasets: ChartDataset[];
}

export interface ChartDataset {
  label: string;
  data: number[];
  borderColor?: string;
  backgroundColor?: string;
  fill?: boolean;
}

export interface TimeSeriesData {
  timestamp: string;
  value: number;
  label?: string;
}

export interface AggregatedMetrics {
  period: 'hour' | 'day' | 'week' | 'month';
  startDate: string;
  endDate: string;
  metrics: {
    [key: string]: {
      avg: number;
      min: number;
      max: number;
      sum: number;
      count: number;
    };
  };
}

export interface Alert {
  id: string;
  type: 'system' | 'agent' | 'network' | 'security' | 'economic';
  severity: 'info' | 'warning' | 'error' | 'critical';
  title: string;
  message: string;
  source: string;
  timestamp: string;
  acknowledged: boolean;
  resolvedAt?: string;
  metadata?: Record<string, any>;
}

export interface Activity {
  id: string;
  type: ActivityType;
  title: string;
  description?: string;
  user?: {
    id: string;
    name: string;
    avatar?: string;
  };
  agent?: {
    id: string;
    name: string;
  };
  timestamp: string;
  metadata?: Record<string, any>;
}

export type ActivityType = 
  | 'agent_created'
  | 'agent_updated'
  | 'agent_deleted'
  | 'customer_onboarded'
  | 'payment_received'
  | 'alert_triggered'
  | 'rule_executed'
  | 'model_deployed'
  | 'system_update';

export interface InfrastructureNode {
  id: string;
  name: string;
  location: {
    lat: number;
    lng: number;
    region: string;
    country: string;
  };
  status: 'online' | 'degraded' | 'offline';
  metrics: {
    agentCount: number;
    cpuUtilization: number;
    memoryUtilization: number;
    networkLatency: number;
    customers: number;
  };
  services: {
    orchestrator: boolean;
    mlTraining: boolean;
    p2pNode: boolean;
    storage: boolean;
  };
}