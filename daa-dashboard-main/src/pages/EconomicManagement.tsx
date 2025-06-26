
import React from 'react';
import { DollarSign, TrendingUp, PieChart, BarChart3, Coins, Wallet } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { ChartContainer, ChartTooltip, ChartTooltipContent } from '@/components/ui/chart';
import { LineChart, Line, XAxis, YAxis, ResponsiveContainer, AreaChart, Area, BarChart, Bar } from 'recharts';
import DashboardLayout from '@/components/DashboardLayout';

const EconomicManagement = () => {
  const revenueData = [
    { month: 'Jan', revenue: 8950000, costs: 6200000, profit: 2750000 },
    { month: 'Feb', revenue: 9250000, costs: 6400000, profit: 2850000 },
    { month: 'Mar', revenue: 9800000, costs: 6600000, profit: 3200000 },
    { month: 'Apr', revenue: 10250000, costs: 6800000, profit: 3450000 },
    { month: 'May', revenue: 10650000, costs: 7000000, profit: 3650000 },
    { month: 'Jun', revenue: 11200000, costs: 7200000, profit: 4000000 }
  ];

  const tokenData = [
    { time: '00:00', price: 1.24, volume: 2400000 },
    { time: '04:00', price: 1.26, volume: 1800000 },
    { time: '08:00', price: 1.28, volume: 3200000 },
    { time: '12:00', price: 1.31, volume: 2800000 },
    { time: '16:00', price: 1.29, volume: 2200000 },
    { time: '20:00', price: 1.32, volume: 2600000 }
  ];

  const costBreakdown = [
    { category: 'Infrastructure', amount: 2800000, percentage: 38.9 },
    { category: 'Personnel', amount: 2200000, percentage: 30.6 },
    { category: 'Operations', amount: 1400000, percentage: 19.4 },
    { category: 'Marketing', amount: 800000, percentage: 11.1 }
  ];

  return (
    <DashboardLayout>
      <div className="p-3 sm:p-6 space-y-4 sm:space-y-6">
        <div className="flex items-center space-x-3">
          <DollarSign className="h-8 w-8 text-green-400" />
          <h1 className="text-3xl font-bold">Economic Management</h1>
          <div className="bg-green-500/20 text-green-400 px-3 py-1 rounded-full text-sm">+15% Growth</div>
        </div>

        {/* Metrics Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">Monthly Revenue</p>
                  <p className="text-2xl font-bold text-green-400">$11.2M</p>
                  <p className="text-green-400 text-sm">+15.3% from last month</p>
                </div>
                <TrendingUp className="h-8 w-8 text-green-400/70" />
              </div>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">rUv Token Price</p>
                  <p className="text-2xl font-bold text-green-400">$1.32</p>
                  <p className="text-green-400 text-sm">+6.5% (24h)</p>
                </div>
                <Coins className="h-8 w-8 text-green-400/70" />
              </div>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">Profit Margin</p>
                  <p className="text-2xl font-bold text-green-400">35.7%</p>
                  <p className="text-green-400 text-sm">+2.1% improvement</p>
                </div>
                <PieChart className="h-8 w-8 text-green-400/70" />
              </div>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">Token Supply</p>
                  <p className="text-2xl font-bold text-green-400">1.2B</p>
                  <p className="text-green-400 text-sm">Fixed supply</p>
                </div>
                <Wallet className="h-8 w-8 text-green-400/70" />
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Charts Grid */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <Card className="bg-gray-900/50 border-green-500/20">
            <CardHeader>
              <CardTitle className="text-green-400">Revenue & Profit Trends</CardTitle>
            </CardHeader>
            <CardContent>
              <ChartContainer config={{ revenue: { color: '#10b981' }, profit: { color: '#3b82f6' } }}>
                <ResponsiveContainer width="100%" height={300}>
                  <AreaChart data={revenueData}>
                    <XAxis dataKey="month" tick={{ fill: '#10b981', fontSize: 12 }} />
                    <YAxis tick={{ fill: '#10b981', fontSize: 12 }} />
                    <ChartTooltip content={<ChartTooltipContent />} />
                    <Area type="monotone" dataKey="revenue" stackId="1" stroke="#10b981" fill="#10b981" fillOpacity={0.3} />
                    <Area type="monotone" dataKey="profit" stackId="2" stroke="#3b82f6" fill="#3b82f6" fillOpacity={0.3} />
                  </AreaChart>
                </ResponsiveContainer>
              </ChartContainer>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardHeader>
              <CardTitle className="text-green-400">rUv Token Performance</CardTitle>
            </CardHeader>
            <CardContent>
              <ChartContainer config={{ price: { color: '#10b981' }, volume: { color: '#6b7280' } }}>
                <ResponsiveContainer width="100%" height={300}>
                  <LineChart data={tokenData}>
                    <XAxis dataKey="time" tick={{ fill: '#10b981', fontSize: 12 }} />
                    <YAxis tick={{ fill: '#10b981', fontSize: 12 }} />
                    <ChartTooltip content={<ChartTooltipContent />} />
                    <Line type="monotone" dataKey="price" stroke="#10b981" strokeWidth={3} />
                  </LineChart>
                </ResponsiveContainer>
              </ChartContainer>
            </CardContent>
          </Card>
        </div>

        {/* Cost Breakdown */}
        <Card className="bg-gray-900/50 border-green-500/20">
          <CardHeader>
            <CardTitle className="text-green-400">Cost Breakdown Analysis</CardTitle>
          </CardHeader>
          <CardContent>
            <ChartContainer config={{ amount: { color: '#10b981' } }}>
              <ResponsiveContainer width="100%" height={300}>
                <BarChart data={costBreakdown}>
                  <XAxis dataKey="category" tick={{ fill: '#10b981', fontSize: 12 }} />
                  <YAxis tick={{ fill: '#10b981', fontSize: 12 }} />
                  <ChartTooltip content={<ChartTooltipContent />} />
                  <Bar dataKey="amount" fill="#10b981" />
                </BarChart>
              </ResponsiveContainer>
            </ChartContainer>
          </CardContent>
        </Card>
      </div>
    </DashboardLayout>
  );
};

export default EconomicManagement;
