import React from 'react';
import { Zap, Brain, Cpu, TrendingUp, BarChart3, Activity } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { ChartContainer, ChartTooltip, ChartTooltipContent } from '@/components/ui/chart';
import { LineChart, Line, XAxis, YAxis, ResponsiveContainer, AreaChart, Area, BarChart, Bar, ScatterChart, Scatter } from 'recharts';
import DashboardLayout from '@/components/DashboardLayout';

const AIMLOperations = () => {
  const modelPerformanceData = [
    { time: '00:00', accuracy: 0.89, latency: 24 },
    { time: '04:00', accuracy: 0.91, latency: 22 },
    { time: '08:00', accuracy: 0.93, latency: 25 },
    { time: '12:00', accuracy: 0.92, latency: 23 },
    { time: '16:00', accuracy: 0.94, latency: 21 },
    { time: '20:00', accuracy: 0.95, latency: 20 }
  ];

  const resourceUtilizationData = [
    { time: '00:00', cpu: 65, memory: 72 },
    { time: '04:00', cpu: 68, memory: 75 },
    { time: '08:00', cpu: 72, memory: 78 },
    { time: '12:00', cpu: 70, memory: 76 },
    { time: '16:00', cpu: 74, memory: 80 },
    { time: '20:00', cpu: 76, memory: 82 }
  ];

  const trainingMetrics = [
    { model: 'Model A', dataSize: 245, trainingTime: 3.2 },
    { model: 'Model B', dataSize: 312, trainingTime: 4.1 },
    { model: 'Model C', dataSize: 198, trainingTime: 2.8 },
    { model: 'Model D', dataSize: 287, trainingTime: 3.7 },
    { model: 'Model E', dataSize: 356, trainingTime: 4.5 }
  ];

  return (
    <DashboardLayout>
      <div className="p-3 sm:p-6 space-y-4 sm:space-y-6">
        <div className="flex items-center space-x-3">
          <Zap className="h-8 w-8 text-green-400" />
          <h1 className="text-3xl font-bold">AI & ML Operations</h1>
          <div className="bg-green-500/20 text-green-400 px-3 py-1 rounded-full text-sm">47 Models</div>
        </div>

        {/* Metrics Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">Active Models</p>
                  <p className="text-2xl font-bold text-green-400">47</p>
                  <p className="text-green-400 text-sm">+4 models this month</p>
                </div>
                <Brain className="h-8 w-8 text-green-400/70" />
              </div>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">Avg. Accuracy</p>
                  <p className="text-2xl font-bold text-green-400">93.4%</p>
                  <p className="text-green-400 text-sm">Stable performance</p>
                </div>
                <TrendingUp className="h-8 w-8 text-green-400/70" />
              </div>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">CPU Utilization</p>
                  <p className="text-2xl font-bold text-green-400">72%</p>
                  <p className="text-green-400 text-sm">Optimized usage</p>
                </div>
                <Cpu className="h-8 w-8 text-green-400/70" />
              </div>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">Model Latency</p>
                  <p className="text-2xl font-bold text-green-400">22ms</p>
                  <p className="text-green-400 text-sm">Low latency</p>
                </div>
                <Activity className="h-8 w-8 text-green-400/70" />
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Charts Grid */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <Card className="bg-gray-900/50 border-green-500/20">
            <CardHeader>
              <CardTitle className="text-green-400">Model Performance Metrics</CardTitle>
            </CardHeader>
            <CardContent>
              <ChartContainer config={{ accuracy: { color: '#10b981' }, latency: { color: '#3b82f6' } }}>
                <ResponsiveContainer width="100%" height={300}>
                  <LineChart data={modelPerformanceData}>
                    <XAxis dataKey="time" tick={{ fill: '#10b981', fontSize: 12 }} />
                    <YAxis tick={{ fill: '#10b981', fontSize: 12 }} />
                    <ChartTooltip content={<ChartTooltipContent />} />
                    <Line type="monotone" dataKey="accuracy" stroke="#10b981" strokeWidth={2} />
                    <Line type="monotone" dataKey="latency" stroke="#3b82f6" strokeWidth={2} />
                  </LineChart>
                </ResponsiveContainer>
              </ChartContainer>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardHeader>
              <CardTitle className="text-green-400">Resource Utilization</CardTitle>
            </CardHeader>
            <CardContent>
              <ChartContainer config={{ cpu: { color: '#10b981' }, memory: { color: '#f59e0b' } }}>
                <ResponsiveContainer width="100%" height={300}>
                  <AreaChart data={resourceUtilizationData}>
                    <XAxis dataKey="time" tick={{ fill: '#10b981', fontSize: 12 }} />
                    <YAxis tick={{ fill: '#10b981', fontSize: 12 }} />
                    <ChartTooltip content={<ChartTooltipContent />} />
                    <Area type="monotone" dataKey="cpu" stackId="1" stroke="#10b981" fill="#10b981" fillOpacity={0.3} />
                    <Area type="monotone" dataKey="memory" stackId="2" stroke="#f59e0b" fill="#f59e0b" fillOpacity={0.3} />
                  </AreaChart>
                </ResponsiveContainer>
              </ChartContainer>
            </CardContent>
          </Card>
        </div>

        {/* Training Metrics */}
        <Card className="bg-gray-900/50 border-green-500/20">
          <CardHeader>
            <CardTitle className="text-green-400">Model Training Metrics</CardTitle>
          </CardHeader>
          <CardContent>
            <ChartContainer config={{ dataSize: { color: '#10b981' }, trainingTime: { color: '#3b82f6' } }}>
              <ResponsiveContainer width="100%" height={300}>
                <BarChart data={trainingMetrics}>
                  <XAxis dataKey="model" tick={{ fill: '#10b981', fontSize: 12 }} />
                  <YAxis tick={{ fill: '#10b981', fontSize: 12 }} />
                  <ChartTooltip content={<ChartTooltipContent />} />
                  <Bar dataKey="dataSize" fill="#10b981" />
                  <Bar dataKey="trainingTime" fill="#3b82f6" />
                </BarChart>
              </ResponsiveContainer>
            </ChartContainer>
          </CardContent>
        </Card>
      </div>
    </DashboardLayout>
  );
};

export default AIMLOperations;
