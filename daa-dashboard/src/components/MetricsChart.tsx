
import React from 'react';
import { LineChart, Line, AreaChart, Area, XAxis, YAxis, CartesianGrid, ResponsiveContainer, BarChart, Bar } from 'recharts';

const MetricsChart = () => {
  const performanceData = [
    { time: '00:00', cpu: 23, memory: 45, network: 67, agents: 1420 },
    { time: '04:00', cpu: 18, memory: 42, network: 72, agents: 1380 },
    { time: '08:00', cpu: 35, memory: 58, network: 85, agents: 1450 },
    { time: '12:00', cpu: 42, memory: 63, network: 91, agents: 1480 },
    { time: '16:00', cpu: 38, memory: 61, network: 88, agents: 1465 },
    { time: '20:00', cpu: 29, memory: 52, network: 79, agents: 1440 },
    { time: '23:59', cpu: 25, memory: 48, network: 74, agents: 1430 },
  ];

  const networkLatencyData = [
    { region: 'US-E', latency: 12 },
    { region: 'US-W', latency: 18 },
    { region: 'EU-C', latency: 25 },
    { region: 'ASIA', latency: 35 },
    { region: 'EU-W', latency: 45 },
    { region: 'INDIA', latency: 28 },
    { region: 'OCE', latency: 52 },
  ];

  return (
    <div className="space-y-6">
      {/* System Performance Over Time */}
      <div className="h-64">
        <h4 className="text-sm text-green-400/70 mb-3 font-mono uppercase tracking-wide">System Performance (24h)</h4>
        <ResponsiveContainer width="100%" height="100%">
          <AreaChart data={performanceData}>
            <defs>
              <linearGradient id="cpuGradient" x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor="#10b981" stopOpacity={0.3}/>
                <stop offset="95%" stopColor="#10b981" stopOpacity={0.05}/>
              </linearGradient>
              <linearGradient id="memoryGradient" x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor="#6b7280" stopOpacity={0.3}/>
                <stop offset="95%" stopColor="#6b7280" stopOpacity={0.05}/>
              </linearGradient>
            </defs>
            <CartesianGrid strokeDasharray="3 3" stroke="#10b981" opacity={0.1} />
            <XAxis 
              dataKey="time" 
              axisLine={false}
              tickLine={false}
              tick={{ fill: '#10b981', fontSize: 11, fontFamily: 'monospace' }}
            />
            <YAxis 
              axisLine={false}
              tickLine={false}
              tick={{ fill: '#10b981', fontSize: 11, fontFamily: 'monospace' }}
            />
            <Area
              type="monotone"
              dataKey="cpu"
              stroke="#10b981"
              strokeWidth={2}
              fill="url(#cpuGradient)"
            />
            <Area
              type="monotone"
              dataKey="memory"
              stroke="#6b7280"
              strokeWidth={2}
              fill="url(#memoryGradient)"
            />
          </AreaChart>
        </ResponsiveContainer>
      </div>

      {/* Network Latency by Region */}
      <div className="h-48">
        <h4 className="text-sm text-green-400/70 mb-3 font-mono uppercase tracking-wide">Network Latency (ms)</h4>
        <ResponsiveContainer width="100%" height="100%">
          <BarChart data={networkLatencyData}>
            <CartesianGrid strokeDasharray="3 3" stroke="#10b981" opacity={0.1} />
            <XAxis 
              dataKey="region" 
              axisLine={false}
              tickLine={false}
              tick={{ fill: '#10b981', fontSize: 11, fontFamily: 'monospace' }}
            />
            <YAxis 
              axisLine={false}
              tickLine={false}
              tick={{ fill: '#10b981', fontSize: 11, fontFamily: 'monospace' }}
            />
            <Bar 
              dataKey="latency" 
              fill="#374151"
              stroke="#10b981"
              strokeWidth={1}
            />
          </BarChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
};

export default MetricsChart;
