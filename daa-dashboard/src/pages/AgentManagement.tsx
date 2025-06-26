
import React from 'react';
import { Bot, Activity, Zap, Shield, TrendingUp, Users } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { ChartContainer, ChartTooltip, ChartTooltipContent } from '@/components/ui/chart';
import { LineChart, Line, XAxis, YAxis, ResponsiveContainer, AreaChart, Area, PieChart, Pie, Cell, BarChart, Bar } from 'recharts';
import DashboardLayout from '@/components/DashboardLayout';

const AgentManagement = () => {
  const agentActivityData = [
    { time: '00:00', active: 247, idle: 123 },
    { time: '04:00', active: 254, idle: 116 },
    { time: '08:00', active: 262, idle: 108 },
    { time: '12:00', active: 271, idle: 99 },
    { time: '16:00', active: 265, idle: 105 },
    { time: '20:00', active: 258, idle: 112 }
  ];

  const taskCompletionData = [
    { category: 'Data Analysis', completed: 120, pending: 30 },
    { category: 'Network Monitoring', completed: 150, pending: 20 },
    { category: 'Security Audits', completed: 90, pending: 10 },
    { category: 'System Updates', completed: 110, pending: 15 }
  ];

  const securityBreachData = [
    { time: '00:00', breaches: 2, resolved: 1 },
    { time: '04:00', breaches: 1, resolved: 1 },
    { time: '08:00', breaches: 3, resolved: 2 },
    { time: '12:00', breaches: 2, resolved: 2 },
    { time: '16:00', breaches: 1, resolved: 0 },
    { time: '20:00', breaches: 2, resolved: 1 }
  ];

  const COLORS = ['#0088FE', '#00C49F', '#FFBB28', '#FF8042'];

  return (
    <DashboardLayout>
      <div className="p-3 sm:p-6 space-y-4 sm:space-y-6">
        <div className="flex items-center space-x-3">
          <Bot className="h-8 w-8 text-green-400" />
          <h1 className="text-3xl font-bold">Agent Management</h1>
          <div className="bg-green-500/20 text-green-400 px-3 py-1 rounded-full text-sm">1,247 Active</div>
        </div>

        {/* Metrics Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">Active Agents</p>
                  <p className="text-2xl font-bold text-green-400">265</p>
                  <p className="text-green-400 text-sm">+3.2% from last week</p>
                </div>
                <Activity className="h-8 w-8 text-green-400/70" />
              </div>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">Tasks Completed</p>
                  <p className="text-2xl font-bold text-green-400">470</p>
                  <p className="text-green-400 text-sm">+8.5% this month</p>
                </div>
                <Zap className="h-8 w-8 text-green-400/70" />
              </div>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">Security Breaches</p>
                  <p className="text-2xl font-bold text-green-400">2</p>
                  <p className="text-green-400 text-sm">All resolved</p>
                </div>
                <Shield className="h-8 w-8 text-green-400/70" />
              </div>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">Overall Efficiency</p>
                  <p className="text-2xl font-bold text-green-400">92.7%</p>
                  <p className="text-green-400 text-sm">Trending Up</p>
                </div>
                <TrendingUp className="h-8 w-8 text-green-400/70" />
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Charts Grid */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <Card className="bg-gray-900/50 border-green-500/20">
            <CardHeader>
              <CardTitle className="text-green-400">Agent Activity</CardTitle>
            </CardHeader>
            <CardContent>
              <ChartContainer config={{ active: { color: '#10b981' }, idle: { color: '#3b82f6' } }}>
                <ResponsiveContainer width="100%" height={300}>
                  <AreaChart data={agentActivityData}>
                    <XAxis dataKey="time" tick={{ fill: '#10b981', fontSize: 12 }} />
                    <YAxis tick={{ fill: '#10b981', fontSize: 12 }} />
                    <ChartTooltip content={<ChartTooltipContent />} />
                    <Area type="monotone" dataKey="active" stackId="1" stroke="#10b981" fill="#10b981" fillOpacity={0.3} />
                    <Area type="monotone" dataKey="idle" stackId="2" stroke="#3b82f6" fill="#3b82f6" fillOpacity={0.3} />
                  </AreaChart>
                </ResponsiveContainer>
              </ChartContainer>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardHeader>
              <CardTitle className="text-green-400">Task Completion Rate</CardTitle>
            </CardHeader>
            <CardContent>
              <ChartContainer config={{ completed: { color: '#10b981' }, pending: { color: '#6b7280' } }}>
                <ResponsiveContainer width="100%" height={300}>
                  <BarChart data={taskCompletionData}>
                    <XAxis dataKey="category" tick={{ fill: '#10b981', fontSize: 12 }} />
                    <YAxis tick={{ fill: '#10b981', fontSize: 12 }} />
                    <ChartTooltip content={<ChartTooltipContent />} />
                    <Bar dataKey="completed" fill="#10b981" />
                    <Bar dataKey="pending" fill="#6b7280" />
                  </BarChart>
                </ResponsiveContainer>
              </ChartContainer>
            </CardContent>
          </Card>
        </div>

        {/* Security Breach Incidents */}
        <Card className="bg-gray-900/50 border-green-500/20">
          <CardHeader>
            <CardTitle className="text-green-400">Security Breach Incidents</CardTitle>
          </CardHeader>
          <CardContent>
            <ChartContainer config={{ breaches: { color: '#ef4444' }, resolved: { color: '#10b981' } }}>
              <ResponsiveContainer width="100%" height={300}>
                <LineChart data={securityBreachData}>
                  <XAxis dataKey="time" tick={{ fill: '#10b981', fontSize: 12 }} />
                  <YAxis tick={{ fill: '#10b981', fontSize: 12 }} />
                  <ChartTooltip content={<ChartTooltipContent />} />
                  <Line type="monotone" dataKey="breaches" stroke="#ef4444" strokeWidth={2} />
                  <Line type="monotone" dataKey="resolved" stroke="#10b981" strokeWidth={2} />
                </LineChart>
              </ResponsiveContainer>
            </ChartContainer>
          </CardContent>
        </Card>
      </div>
    </DashboardLayout>
  );
};

export default AgentManagement;
