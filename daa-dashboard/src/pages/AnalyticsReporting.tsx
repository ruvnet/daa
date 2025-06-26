
import React from 'react';
import { TrendingUp, BarChart3, PieChart, LineChart, FileText, Download } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { ChartContainer, ChartTooltip, ChartTooltipContent } from '@/components/ui/chart';
import { LineChart as RechartsLineChart, Line, XAxis, YAxis, ResponsiveContainer, AreaChart, Area, BarChart, Bar, PieChart as RechartsPieChart, Pie, Cell } from 'recharts';
import DashboardLayout from '@/components/DashboardLayout';

const AnalyticsReporting = () => {
  const revenueData = [
    { month: 'Jan', revenue: 8950000 },
    { month: 'Feb', revenue: 9250000 },
    { month: 'Mar', revenue: 9800000 },
    { month: 'Apr', revenue: 10250000 },
    { month: 'May', revenue: 10650000 },
    { month: 'Jun', revenue: 11200000 }
  ];

  const customerAcquisitionData = [
    { month: 'Jan', newCustomers: 245 },
    { month: 'Feb', newCustomers: 260 },
    { month: 'Mar', newCustomers: 280 },
    { month: 'Apr', newCustomers: 305 },
    { month: 'May', newCustomers: 320 },
    { month: 'Jun', newCustomers: 335 }
  ];

  const productPerformanceData = [
    { product: 'Alpha', sales: 4500 },
    { product: 'Beta', sales: 3800 },
    { product: 'Gamma', sales: 2900 },
    { product: 'Delta', sales: 1800 }
  ];

  const COLORS = ['#0088FE', '#00C49F', '#FFBB28', '#FF8042'];

  return (
    <DashboardLayout>
      <div className="p-3 sm:p-6 space-y-4 sm:space-y-6">
        <div className="flex items-center space-x-3">
          <TrendingUp className="h-8 w-8 text-green-400" />
          <h1 className="text-3xl font-bold">Analytics & Reporting</h1>
          <div className="bg-green-500/20 text-green-400 px-3 py-1 rounded-full text-sm">47 Reports</div>
        </div>

        {/* Charts Grid */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <Card className="bg-gray-900/50 border-green-500/20">
            <CardHeader>
              <CardTitle className="text-green-400">Monthly Revenue Trend</CardTitle>
            </CardHeader>
            <CardContent>
              <ChartContainer config={{ revenue: { color: '#10b981' } }}>
                <ResponsiveContainer width="100%" height={300}>
                  <RechartsLineChart data={revenueData}>
                    <XAxis dataKey="month" tick={{ fill: '#10b981', fontSize: 12 }} />
                    <YAxis tick={{ fill: '#10b981', fontSize: 12 }} />
                    <ChartTooltip content={<ChartTooltipContent />} />
                    <Line type="monotone" dataKey="revenue" stroke="#10b981" strokeWidth={2} />
                  </RechartsLineChart>
                </ResponsiveContainer>
              </ChartContainer>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardHeader>
              <CardTitle className="text-green-400">Customer Acquisition</CardTitle>
            </CardHeader>
            <CardContent>
              <ChartContainer config={{ newCustomers: { color: '#3b82f6' } }}>
                <ResponsiveContainer width="100%" height={300}>
                  <AreaChart data={customerAcquisitionData}>
                    <XAxis dataKey="month" tick={{ fill: '#10b981', fontSize: 12 }} />
                    <YAxis tick={{ fill: '#10b981', fontSize: 12 }} />
                    <ChartTooltip content={<ChartTooltipContent />} />
                    <Area type="monotone" dataKey="newCustomers" stroke="#3b82f6" fill="#3b82f6" fillOpacity={0.3} />
                  </AreaChart>
                </ResponsiveContainer>
              </ChartContainer>
            </CardContent>
          </Card>
        </div>

        {/* Product Performance */}
        <Card className="bg-gray-900/50 border-green-500/20">
          <CardHeader>
            <CardTitle className="text-green-400">Product Performance Analysis</CardTitle>
          </CardHeader>
          <CardContent>
            <ChartContainer config={{ sales: { color: '#8884d8' } }}>
              <ResponsiveContainer width="100%" height={300}>
                <RechartsPieChart>
                  <Pie
                    dataKey="sales"
                    data={productPerformanceData}
                    cx="50%"
                    cy="50%"
                    outerRadius={80}
                    fill="#8884d8"
                    label
                  >
                    {
                      productPerformanceData.map((entry, index) => (
                        <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                      ))
                    }
                  </Pie>
                  <ChartTooltip content={<ChartTooltipContent />} />
                </RechartsPieChart>
              </ResponsiveContainer>
            </ChartContainer>
          </CardContent>
        </Card>

        {/* Report Generation */}
        <Card className="bg-gray-900/50 border-green-500/20">
          <CardHeader>
            <CardTitle className="text-green-400">Generate Reports</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center justify-between">
              <p className="text-green-400/70">Generate a detailed report for the last quarter.</p>
              <button className="bg-green-500/10 text-green-400 px-3 py-2 rounded-md text-sm hover:bg-green-500/20 transition-colors">
                <Download className="h-4 w-4 mr-2 inline-block" />
                Download Report
              </button>
            </div>
          </CardContent>
        </Card>
      </div>
    </DashboardLayout>
  );
};

export default AnalyticsReporting;
