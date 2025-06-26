
import React, { useState, useEffect } from 'react';
import { 
  Activity, 
  Globe, 
  TrendingUp, 
  Users, 
  Server, 
  DollarSign,
  Bot,
  Network,
  Shield
} from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import GlobalMap from '@/components/GlobalMap';
import MetricsChart from '@/components/MetricsChart';
import ActivityFeed from '@/components/ActivityFeed';
import NetworkTopology from '@/components/NetworkTopology';
import AlertsPanel from '@/components/AlertsPanel';
import DashboardLayout from '@/components/DashboardLayout';

const Index = () => {
  // Real-time metrics simulation
  const [metrics, setMetrics] = useState({
    totalAgents: 145672,
    activeCustomers: 2847,
    monthlyRevenue: 8950000,
    systemUptime: 99.97,
    networkNodes: 1247,
    securityAlerts: 3
  });

  useEffect(() => {
    const timer = setInterval(() => {
      // Simulate real-time metric updates
      setMetrics(prev => ({
        ...prev,
        totalAgents: prev.totalAgents + Math.floor(Math.random() * 10) - 5,
        networkNodes: prev.networkNodes + Math.floor(Math.random() * 6) - 3,
        securityAlerts: Math.max(0, prev.securityAlerts + Math.floor(Math.random() * 3) - 1)
      }));
    }, 5000);

    return () => clearInterval(timer);
  }, []);

  const MetricCard = ({ title, value, change, trend, icon: Icon, suffix = '', prefix = '' }) => (
    <Card className="bg-gray-900/50 border-green-500/20 hover:border-green-500/40 transition-all duration-300">
      <CardContent className="p-4 sm:p-6">
        <div className="flex items-center justify-between">
          <div className="flex-1 min-w-0">
            <p className="text-green-400/70 text-xs sm:text-sm font-mono uppercase tracking-wide truncate">{title}</p>
            <p className="text-lg sm:text-2xl font-bold text-green-400 font-mono mt-1 truncate">
              {prefix}{typeof value === 'number' ? value.toLocaleString() : value}{suffix}
            </p>
            {change && (
              <div className={`flex items-center mt-1 sm:mt-2 text-xs sm:text-sm font-mono ${trend === 'up' ? 'text-green-400' : trend === 'down' ? 'text-red-400' : 'text-gray-400'}`}>
                <span className="truncate">{change}</span>
              </div>
            )}
          </div>
          <div className="bg-green-500/10 p-2 sm:p-3 rounded-lg ml-2 flex-shrink-0">
            <Icon className="h-5 w-5 sm:h-8 sm:w-8 text-green-400" />
          </div>
        </div>
      </CardContent>
    </Card>
  );

  return (
    <DashboardLayout>
      <div className="p-3 sm:p-6 space-y-4 sm:space-y-6">
        {/* Hero Metrics */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-6 gap-3 sm:gap-6">
          <MetricCard 
            title="Total Agents" 
            value={metrics.totalAgents} 
            change="+2.4%" 
            trend="up" 
            icon={Bot} 
          />
          <MetricCard 
            title="Active Customers" 
            value={metrics.activeCustomers} 
            change="+8.1%" 
            trend="up" 
            icon={Users} 
          />
          <MetricCard 
            title="Monthly Revenue" 
            value={metrics.monthlyRevenue} 
            change="+15.3%" 
            trend="up" 
            icon={DollarSign} 
            prefix="$" 
          />
          <MetricCard 
            title="System Uptime" 
            value={metrics.systemUptime} 
            change="99.97%" 
            trend="up"
            icon={Server} 
            suffix="%" 
          />
          <MetricCard 
            title="Network Nodes" 
            value={metrics.networkNodes} 
            change="+5.2%" 
            trend="up" 
            icon={Network} 
          />
          <MetricCard 
            title="Security Status" 
            value={metrics.securityAlerts === 0 ? "SECURE" : `${metrics.securityAlerts} ALERTS`} 
            change={metrics.securityAlerts === 0 ? "No threats" : "Active monitoring"} 
            trend={metrics.securityAlerts === 0 ? "up" : "down"} 
            icon={Shield} 
          />
        </div>

        {/* Global Infrastructure Map */}
        <Card className="bg-gray-900/50 border-green-500/20">
          <CardHeader className="pb-3 sm:pb-6">
            <CardTitle className="text-green-400 flex items-center text-sm sm:text-base">
              <Globe className="h-4 w-4 sm:h-5 sm:w-5 mr-2" />
              Global Infrastructure Status
            </CardTitle>
          </CardHeader>
          <CardContent className="p-3 sm:p-6 pt-0">
            <GlobalMap />
          </CardContent>
        </Card>

        {/* Charts and Analytics */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 sm:gap-6">
          <Card className="bg-gray-900/50 border-green-500/20">
            <CardHeader className="pb-3 sm:pb-6">
              <CardTitle className="text-green-400 flex items-center text-sm sm:text-base">
                <TrendingUp className="h-4 w-4 sm:h-5 sm:w-5 mr-2" />
                Performance Metrics
              </CardTitle>
            </CardHeader>
            <CardContent className="p-3 sm:p-6 pt-0">
              <MetricsChart />
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardHeader className="pb-3 sm:pb-6">
              <CardTitle className="text-green-400 flex items-center text-sm sm:text-base">
                <Network className="h-4 w-4 sm:h-5 sm:w-5 mr-2" />
                Network Topology
              </CardTitle>
            </CardHeader>
            <CardContent className="p-3 sm:p-6 pt-0">
              <NetworkTopology />
            </CardContent>
          </Card>
        </div>

        {/* Activity Feed and Alerts */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-4 sm:gap-6">
          <div className="lg:col-span-2">
            <ActivityFeed />
          </div>
          <div>
            <AlertsPanel />
          </div>
        </div>
      </div>
    </DashboardLayout>
  );
};

export default Index;
