import React from 'react';
import { Users, UserPlus, TrendingUp, DollarSign, Heart, AlertCircle } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { ChartContainer, ChartTooltip, ChartTooltipContent } from '@/components/ui/chart';
import { LineChart, Line, XAxis, YAxis, ResponsiveContainer, AreaChart, Area, BarChart, Bar, PieChart, Pie, Cell } from 'recharts';
import DashboardLayout from '@/components/DashboardLayout';

const CustomerManagement = () => {
  const customerGrowth = [
    { month: 'Jan', total: 2847, new: 124, churned: 18 },
    { month: 'Feb', total: 2953, new: 136, churned: 30 },
    { month: 'Mar', total: 3089, new: 148, churned: 12 },
    { month: 'Apr', total: 3234, new: 156, churned: 11 },
    { month: 'May', total: 3398, new: 178, churned: 14 },
    { month: 'Jun', total: 3567, new: 185, churned: 16 }
  ];

  const customerTiers = [
    { name: 'Enterprise', count: 89, revenue: 8950000, color: '#10b981' },
    { name: 'Professional', count: 234, revenue: 2340000, color: '#3b82f6' },
    { name: 'Standard', count: 567, revenue: 1134000, color: '#f59e0b' },
    { name: 'Basic', count: 2677, revenue: 534000, color: '#6b7280' }
  ];

  const supportMetrics = [
    { time: '00:00', tickets: 23, resolved: 21, satisfaction: 4.2 },
    { time: '04:00', tickets: 18, resolved: 17, satisfaction: 4.3 },
    { time: '08:00', tickets: 45, resolved: 42, satisfaction: 4.1 },
    { time: '12:00', tickets: 67, resolved: 61, satisfaction: 4.0 },
    { time: '16:00', tickets: 52, resolved: 48, satisfaction: 4.2 },
    { time: '20:00', tickets: 34, resolved: 32, satisfaction: 4.4 }
  ];

  const healthScores = [
    { segment: 'High Value', health: 92, risk: 8 },
    { segment: 'Growing', health: 85, risk: 15 },
    { segment: 'Stable', health: 78, risk: 22 },
    { segment: 'At Risk', health: 65, risk: 35 },
    { segment: 'Critical', health: 45, risk: 55 }
  ];

  return (
    <DashboardLayout>
      <div className="p-3 sm:p-6 space-y-4 sm:space-y-6">
        <div className="flex items-center space-x-3">
          <Users className="h-8 w-8 text-green-400" />
          <h1 className="text-3xl font-bold">Customer Management</h1>
          <div className="bg-green-500/20 text-green-400 px-3 py-1 rounded-full text-sm">2,847 Customers</div>
        </div>

        {/* Metrics Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">Total Customers</p>
                  <p className="text-2xl font-bold text-green-400">3,567</p>
                  <p className="text-green-400 text-sm">+4.7% this month</p>
                </div>
                <UserPlus className="h-8 w-8 text-green-400/70" />
              </div>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">Customer LTV</p>
                  <p className="text-2xl font-bold text-green-400">$48.2K</p>
                  <p className="text-green-400 text-sm">+12.3% increase</p>
                </div>
                <DollarSign className="h-8 w-8 text-green-400/70" />
              </div>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">Satisfaction</p>
                  <p className="text-2xl font-bold text-green-400">4.2/5</p>
                  <p className="text-green-400 text-sm">Above target</p>
                </div>
                <Heart className="h-8 w-8 text-green-400/70" />
              </div>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">Churn Rate</p>
                  <p className="text-2xl font-bold text-yellow-400">2.3%</p>
                  <p className="text-yellow-400 text-sm">Within target</p>
                </div>
                <AlertCircle className="h-8 w-8 text-yellow-400/70" />
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Charts Grid */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <Card className="bg-gray-900/50 border-green-500/20">
            <CardHeader>
              <CardTitle className="text-green-400">Customer Growth Trends</CardTitle>
            </CardHeader>
            <CardContent>
              <ChartContainer config={{ total: { color: '#10b981' }, new: { color: '#3b82f6' }, churned: { color: '#ef4444' } }}>
                <ResponsiveContainer width="100%" height={300}>
                  <LineChart data={customerGrowth}>
                    <XAxis dataKey="month" tick={{ fill: '#10b981', fontSize: 12 }} />
                    <YAxis tick={{ fill: '#10b981', fontSize: 12 }} />
                    <ChartTooltip content={<ChartTooltipContent />} />
                    <Line type="monotone" dataKey="total" stroke="#10b981" strokeWidth={3} />
                    <Line type="monotone" dataKey="new" stroke="#3b82f6" strokeWidth={2} />
                    <Line type="monotone" dataKey="churned" stroke="#ef4444" strokeWidth={2} />
                  </LineChart>
                </ResponsiveContainer>
              </ChartContainer>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardHeader>
              <CardTitle className="text-green-400">Customer Tier Distribution</CardTitle>
            </CardHeader>
            <CardContent>
              <ChartContainer config={{}}>
                <ResponsiveContainer width="100%" height={300}>
                  <PieChart>
                    <Pie
                      data={customerTiers}
                      cx="50%"
                      cy="50%"
                      outerRadius={100}
                      dataKey="count"
                      label={({ name, count }) => `${name}: ${count}`}
                    >
                      {customerTiers.map((entry, index) => (
                        <Cell key={`cell-${index}`} fill={entry.color} />
                      ))}
                    </Pie>
                    <ChartTooltip />
                  </PieChart>
                </ResponsiveContainer>
              </ChartContainer>
            </CardContent>
          </Card>
        </div>

        {/* Support and Health Metrics */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <Card className="bg-gray-900/50 border-green-500/20">
            <CardHeader>
              <CardTitle className="text-green-400">Support Metrics</CardTitle>
            </CardHeader>
            <CardContent>
              <ChartContainer config={{ tickets: { color: '#f59e0b' }, resolved: { color: '#10b981' } }}>
                <ResponsiveContainer width="100%" height={300}>
                  <AreaChart data={supportMetrics}>
                    <XAxis dataKey="time" tick={{ fill: '#10b981', fontSize: 12 }} />
                    <YAxis tick={{ fill: '#10b981', fontSize: 12 }} />
                    <ChartTooltip content={<ChartTooltipContent />} />
                    <Area type="monotone" dataKey="tickets" stackId="1" stroke="#f59e0b" fill="#f59e0b" fillOpacity={0.3} />
                    <Area type="monotone" dataKey="resolved" stackId="2" stroke="#10b981" fill="#10b981" fillOpacity={0.6} />
                  </AreaChart>
                </ResponsiveContainer>
              </ChartContainer>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardHeader>
              <CardTitle className="text-green-400">Customer Health Scores</CardTitle>
            </CardHeader>
            <CardContent>
              <ChartContainer config={{ health: { color: '#10b981' }, risk: { color: '#ef4444' } }}>
                <ResponsiveContainer width="100%" height={300}>
                  <BarChart data={healthScores}>
                    <XAxis dataKey="segment" tick={{ fill: '#10b981', fontSize: 12 }} />
                    <YAxis tick={{ fill: '#10b981', fontSize: 12 }} />
                    <ChartTooltip content={<ChartTooltipContent />} />
                    <Bar dataKey="health" fill="#10b981" />
                    <Bar dataKey="risk" fill="#ef4444" />
                  </BarChart>
                </ResponsiveContainer>
              </ChartContainer>
            </CardContent>
          </Card>
        </div>
      </div>
    </DashboardLayout>
  );
};

export default CustomerManagement;
