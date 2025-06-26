import React from 'react';
import { Server, Settings, HardDrive, Cpu, Database, Wifi } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { ChartContainer, ChartTooltip, ChartTooltipContent } from '@/components/ui/chart';
import { LineChart, Line, XAxis, YAxis, ResponsiveContainer, AreaChart, Area, BarChart, Bar } from 'recharts';
import DashboardLayout from '@/components/DashboardLayout';

const SystemAdministration = () => {
  const systemMetrics = [
    { time: '00:00', cpuUsage: 65, memoryUsage: 78, diskUsage: 54 },
    { time: '04:00', cpuUsage: 68, memoryUsage: 80, diskUsage: 56 },
    { time: '08:00', cpuUsage: 72, memoryUsage: 82, diskUsage: 58 },
    { time: '12:00', cpuUsage: 75, memoryUsage: 85, diskUsage: 60 },
    { time: '16:00', cpuUsage: 70, memoryUsage: 81, diskUsage: 57 },
    { time: '20:00', cpuUsage: 67, memoryUsage: 79, diskUsage: 55 }
  ];

  const serverStatus = [
    { server: 'Web Server 1', status: 'Online', cpuLoad: 72, memoryUsage: 80 },
    { server: 'Database Server', status: 'Online', cpuLoad: 85, memoryUsage: 92 },
    { server: 'Cache Server', status: 'Online', cpuLoad: 60, memoryUsage: 70 },
    { server: 'API Server', status: 'Offline', cpuLoad: 0, memoryUsage: 0 }
  ];

  const networkTraffic = [
    { time: '00:00', inbound: 2.3, outbound: 1.8 },
    { time: '04:00', inbound: 2.5, outbound: 2.0 },
    { time: '08:00', inbound: 2.8, outbound: 2.2 },
    { time: '12:00', inbound: 3.1, outbound: 2.5 },
    { time: '16:00', inbound: 2.7, outbound: 2.1 },
    { time: '20:00', inbound: 2.4, outbound: 1.9 }
  ];

  return (
    <DashboardLayout>
      <div className="p-3 sm:p-6 space-y-4 sm:space-y-6">
        <div className="flex items-center space-x-3">
          <Server className="h-8 w-8 text-green-400" />
          <h1 className="text-3xl font-bold">System Administration</h1>
          <div className="bg-green-500/20 text-green-400 px-3 py-1 rounded-full text-sm">99.97% Uptime</div>
        </div>

        {/* Metrics Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">CPU Usage</p>
                  <p className="text-2xl font-bold text-green-400">71%</p>
                  <p className="text-green-400 text-sm">Normal load</p>
                </div>
                <Cpu className="h-8 w-8 text-green-400/70" />
              </div>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">Memory Usage</p>
                  <p className="text-2xl font-bold text-green-400">81 GB</p>
                  <p className="text-green-400 text-sm">Optimal</p>
                </div>
                <HardDrive className="h-8 w-8 text-green-400/70" />
              </div>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">Disk Usage</p>
                  <p className="text-2xl font-bold text-green-400">57%</p>
                  <p className="text-green-400 text-sm">Capacity OK</p>
                </div>
                <Database className="h-8 w-8 text-green-400/70" />
              </div>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">Network Stability</p>
                  <p className="text-2xl font-bold text-green-400">STABLE</p>
                  <p className="text-green-400 text-sm">99.8% uptime</p>
                </div>
                <Wifi className="h-8 w-8 text-green-400/70" />
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Charts Grid */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <Card className="bg-gray-900/50 border-green-500/20">
            <CardHeader>
              <CardTitle className="text-green-400">System Resource Monitoring</CardTitle>
            </CardHeader>
            <CardContent>
              <ChartContainer config={{ cpuUsage: { color: '#10b981' }, memoryUsage: { color: '#3b82f6' }, diskUsage: { color: '#f59e0b' } }}>
                <ResponsiveContainer width="100%" height={300}>
                  <LineChart data={systemMetrics}>
                    <XAxis dataKey="time" tick={{ fill: '#10b981', fontSize: 12 }} />
                    <YAxis tick={{ fill: '#10b981', fontSize: 12 }} />
                    <ChartTooltip content={<ChartTooltipContent />} />
                    <Line type="monotone" dataKey="cpuUsage" stroke="#10b981" strokeWidth={2} />
                    <Line type="monotone" dataKey="memoryUsage" stroke="#3b82f6" strokeWidth={2} />
                    <Line type="monotone" dataKey="diskUsage" stroke="#f59e0b" strokeWidth={2} />
                  </LineChart>
                </ResponsiveContainer>
              </ChartContainer>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardHeader>
              <CardTitle className="text-green-400">Server Status Overview</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="overflow-x-auto">
                <table className="w-full">
                  <thead>
                    <tr className="text-left">
                      <th className="py-2 text-green-400/70">Server</th>
                      <th className="py-2 text-green-400/70">Status</th>
                      <th className="py-2 text-green-400/70">CPU Load</th>
                      <th className="py-2 text-green-400/70">Memory Usage</th>
                    </tr>
                  </thead>
                  <tbody>
                    {serverStatus.map((server, index) => (
                      <tr key={index} className="border-b border-green-500/20">
                        <td className="py-3">{server.server}</td>
                        <td className="py-3">{server.status}</td>
                        <td className="py-3">{server.cpuLoad}%</td>
                        <td className="py-3">{server.memoryUsage}%</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Network Traffic Analysis */}
        <Card className="bg-gray-900/50 border-green-500/20">
          <CardHeader>
            <CardTitle className="text-green-400">Network Traffic Analysis</CardTitle>
          </CardHeader>
          <CardContent>
            <ChartContainer config={{ inbound: { color: '#10b981' }, outbound: { color: '#3b82f6' } }}>
              <ResponsiveContainer width="100%" height={300}>
                <AreaChart data={networkTraffic}>
                  <XAxis dataKey="time" tick={{ fill: '#10b981', fontSize: 12 }} />
                  <YAxis tick={{ fill: '#10b981', fontSize: 12 }} />
                  <ChartTooltip content={<ChartTooltipContent />} />
                  <Area type="monotone" dataKey="inbound" stackId="1" stroke="#10b981" fill="#10b981" fillOpacity={0.3} />
                  <Area type="monotone" dataKey="outbound" stackId="2" stroke="#3b82f6" fill="#3b82f6" fillOpacity={0.3} />
                </AreaChart>
              </ResponsiveContainer>
            </ChartContainer>
          </CardContent>
        </Card>
      </div>
    </DashboardLayout>
  );
};

export default SystemAdministration;
