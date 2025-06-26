'use client';

import { Card } from '@/components/ui/card';
import { ArrowUpIcon, ArrowDownIcon, UsersIcon, ServerIcon, DollarSignIcon, ShieldCheckIcon, NetworkIcon, AlertCircleIcon } from 'lucide-react';
import { cn } from '@/lib/utils';

interface MetricCardProps {
  title: string;
  value: string | number;
  change?: number;
  trend?: 'up' | 'down';
  icon: React.ReactNode;
  status?: 'healthy' | 'warning' | 'critical';
}

function MetricCard({ title, value, change, trend, icon, status = 'healthy' }: MetricCardProps) {
  const statusColors = {
    healthy: 'text-success',
    warning: 'text-warning',
    critical: 'text-destructive',
  };

  return (
    <Card className="p-6">
      <div className="flex items-center justify-between">
        <div className="space-y-2">
          <p className="text-sm font-medium text-muted-foreground">{title}</p>
          <div className="flex items-baseline gap-2">
            <p className="text-2xl font-bold">{value}</p>
            {change !== undefined && (
              <div className={cn('flex items-center text-sm', trend === 'up' ? 'text-success' : 'text-destructive')}>
                {trend === 'up' ? <ArrowUpIcon className="h-3 w-3" /> : <ArrowDownIcon className="h-3 w-3" />}
                <span>{Math.abs(change)}%</span>
              </div>
            )}
          </div>
        </div>
        <div className={cn('h-12 w-12 rounded-lg bg-muted flex items-center justify-center', statusColors[status])}>
          {icon}
        </div>
      </div>
    </Card>
  );
}

export function HeroMetrics() {
  // Mock data - replace with real data from API
  const metrics = [
    {
      title: 'Total Agents',
      value: '12,453',
      change: 12.5,
      trend: 'up' as const,
      icon: <ServerIcon className="h-6 w-6" />,
      status: 'healthy' as const,
    },
    {
      title: 'Active Customers',
      value: '3,721',
      change: 8.3,
      trend: 'up' as const,
      icon: <UsersIcon className="h-6 w-6" />,
      status: 'healthy' as const,
    },
    {
      title: 'Monthly Revenue',
      value: '$2.4M',
      change: 15.2,
      trend: 'up' as const,
      icon: <DollarSignIcon className="h-6 w-6" />,
      status: 'healthy' as const,
    },
    {
      title: 'System Uptime',
      value: '99.98%',
      change: 0.02,
      trend: 'up' as const,
      icon: <ShieldCheckIcon className="h-6 w-6" />,
      status: 'healthy' as const,
    },
    {
      title: 'Network Nodes',
      value: '847',
      change: 3.1,
      trend: 'up' as const,
      icon: <NetworkIcon className="h-6 w-6" />,
      status: 'healthy' as const,
    },
    {
      title: 'Security Status',
      value: 'Secure',
      icon: <AlertCircleIcon className="h-6 w-6" />,
      status: 'healthy' as const,
    },
  ];

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-6 gap-4">
      {metrics.map((metric, index) => (
        <MetricCard key={index} {...metric} />
      ))}
    </div>
  );
}