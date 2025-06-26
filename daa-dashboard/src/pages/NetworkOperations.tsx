
import React from 'react';
import { Network, Globe, Shield, Wifi, Router, Activity } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { ChartContainer, ChartTooltip, ChartTooltipContent } from '@/components/ui/chart';
import { LineChart, Line, XAxis, YAxis, ResponsiveContainer, AreaChart, Area, ScatterChart, Scatter } from 'recharts';
import DashboardLayout from '@/components/DashboardLayout';

const NetworkOperations = () => {
  const networkMetrics = [
    { time: '00:00', latency: 23, throughput: 2.3, nodes: 1247 },
    { time: '04:00', latency: 21, throughput: 2.1, nodes: 1251 },
    { time: '08:00', latency: 28, throughput: 2.8, nodes: 1249 },
    { time: '12:00', latency: 25, throughput: 2.6, nodes: 1253 },
    { time: '16:00', latency: 22, throughput: 2.4, nodes: 1255 },
    { time: '20:00', latency: 24, throughput: 2.5, nodes: 1258 }
  ];

  const p2pConnections = [
    { region: 'North America', connections: 456, quality: 98.5 },
    { region: 'Europe', connections: 398, quality: 97.8 },
    { region: 'Asia Pacific', connections: 234, quality: 96.9 },
    { region: 'South America', connections: 89, quality: 95.2 },
    { region: 'Africa', connections: 67, quality: 94.8 },
    { region: 'Oceania', connections: 23, quality: 97.1 }
  ];

  const securityEvents = [
    { time: '00:00', threats: 12, blocked: 11, severity: 2 },
    { time: '04:00', threats: 8, blocked: 8, severity: 1 },
    { time: '08:00', threats: 15, blocked: 14, severity: 3 },
    { time: '12:00', threats: 21, blocked: 19, severity: 4 },
    { time: '16:00', threats: 18, blocked: 17, severity: 2 },
    { time: '20:00', threats: 13, blocked: 13, severity: 1 }
  ];

  return (
    <DashboardLayout>
      <div className="p-3 sm:p-6 space-y-4 sm:space-y-6">
        <div className="flex items-center space-x-3">
          <Network className="h-8 w-8 text-green-400" />
          <h1 className="text-3xl font-bold">Network Operations</h1>
          <div className="bg-green-500/20 text-green-400 px-3 py-1 rounded-full text-sm">1,258 Nodes</div>
        </div>

        {/* Metrics Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">Network Nodes</p>
                  <p className="text-2xl font-bold text-green-400">1,258</p>
                  <p className="text-green-400 text-sm">+0.8% growth</p>
                </div>
                <Globe className="h-8 w-8 text-green-400/70" />
              </div>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">Avg Latency</p>
                  <p className="text-2xl font-bold text-green-400">24ms</p>
                  <p className="text-green-400 text-sm">Excellent</p>
                </div>
                <Wifi className="h-8 w-8 text-green-400/70" />
              </div>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">Throughput</p>
                  <p className="text-2xl font-bold text-green-400">2.5 GB/s</p>
                  <p className="text-green-400 text-sm">Peak efficiency</p>
                </div>
                <Activity className="h-8 w-8 text-green-400/70" />
              </div>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">Security Status</p>
                  <p className="text-2xl font-bold text-green-400">SECURE</p>
                  <p className="text-green-400 text-sm">99.2% blocked</p>
                </div>
                <Shield className="h-8 w-8 text-green-400/70" />
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Charts Grid */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <Card className="bg-gray-900/50 border-green-500/20">
            <CardHeader>
              <CardTitle className="text-green-400">Network Performance Metrics</CardTitle>
            </CardHeader>
            <CardContent>
              <ChartContainer config={{ latency: { color: '#10b981' }, throughput: { color: '#3b82f6' } }}>
                <ResponsiveContainer width="100%" height={300}>
                  <LineChart data={networkMetrics}>
                    <XAxis dataKey="time" tick={{ fill: '#10b981', fontSize: 12 }} />
                    <YAxis tick={{ fill: '#10b981', fontSize: 12 }} />
                    <ChartTooltip content={<ChartTooltipContent />} />
                    <Line type="monotone" dataKey="latency" stroke="#10b981" strokeWidth={2} />
                    <Line type="monotone" dataKey="throughput" stroke="#3b82f6" strokeWidth={2} />
                  </LineChart>
                </ResponsiveContainer>
              </ChartContainer>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardHeader>
              <CardTitle className="text-green-400">P2P Network Distribution</CardTitle>
            </CardHeader>
            <CardContent>
              <ChartContainer config={{ connections: { color: '#10b981' }, quality: { color: '#f59e0b' } }}>
                <ResponsiveContainer width="100%" height={300}>
                  <ScatterChart data={p2pConnections}>
                    <XAxis dataKey="connections" tick={{ fill: '#10b981', fontSize: 12 }} />
                    <YAxis dataKey="quality" tick={{ fill: '#10b981', fontSize: 12 }} />
                    <ChartTooltip content={<ChartTooltipContent />} />
                    <Scatter dataKey="quality" fill="#10b981" />
                  </ScatterChart>
                </ResponsiveContainer>
              </ChartContainer>
            </CardContent>
          </Card>
        </div>

        {/* Security Events */}
        <Card className="bg-gray-900/50 border-green-500/20">
          <CardHeader>
            <CardTitle className="text-green-400">Security Events & Threat Detection</CardTitle>
          </CardHeader>
          <CardContent>
            <ChartContainer config={{ threats: { color: '#ef4444' }, blocked: { color: '#10b981' } }}>
              <ResponsiveContainer width="100%" height={300}>
                <AreaChart data={securityEvents}>
                  <XAxis dataKey="time" tick={{ fill: '#10b981', fontSize: 12 }} />
                  <YAxis tick={{ fill: '#10b981', fontSize: 12 }} />
                  <ChartTooltip content={<ChartTooltipContent />} />
                  <Area type="monotone" dataKey="threats" stackId="1" stroke="#ef4444" fill="#ef4444" fillOpacity={0.3} />
                  <Area type="monotone" dataKey="blocked" stackId="2" stroke="#10b981" fill="#10b981" fillOpacity={0.6} />
                </AreaChart>
              </ResponsiveContainer>
            </ChartContainer>
          </CardContent>
        </Card>
      </div>
    </DashboardLayout>
  );
};

export default NetworkOperations;
