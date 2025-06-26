import React from 'react';
import { Shield, Scale, FileText, AlertTriangle, CheckCircle, XCircle } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { ChartContainer, ChartTooltip, ChartTooltipContent } from '@/components/ui/chart';
import { BarChart, Bar, XAxis, YAxis, ResponsiveContainer, LineChart, Line, PieChart, Pie, Cell } from 'recharts';
import DashboardLayout from '@/components/DashboardLayout';

const GovernanceRules = () => {
  const ruleCategories = [
    { category: 'Governance', active: 45, pending: 3, violations: 1 },
    { category: 'Compliance', active: 38, pending: 5, violations: 2 },
    { category: 'Security', active: 52, pending: 2, violations: 0 },
    { category: 'Economic', active: 29, pending: 4, violations: 3 },
    { category: 'Operational', active: 34, pending: 1, violations: 1 }
  ];

  const complianceStatus = [
    { name: 'GDPR', status: 'Compliant', score: 98 },
    { name: 'SOX', status: 'Compliant', score: 95 },
    { name: 'PCI', status: 'Compliant', score: 99 },
    { name: 'ISO27001', status: 'Review', score: 88 },
    { name: 'Custom', status: 'Compliant', score: 92 }
  ];

  const auditTrail = [
    { time: '00:00', actions: 245, violations: 2, reviews: 12 },
    { time: '04:00', actions: 189, violations: 1, reviews: 8 },
    { time: '08:00', actions: 567, violations: 5, reviews: 23 },
    { time: '12:00', actions: 634, violations: 3, reviews: 18 },
    { time: '16:00', actions: 489, violations: 2, reviews: 15 },
    { time: '20:00', actions: 356, violations: 1, reviews: 11 }
  ];

  const statusColors = {
    'Compliant': '#10b981',
    'Review': '#f59e0b',
    'Non-Compliant': '#ef4444'
  };

  return (
    <DashboardLayout>
      <div className="p-3 sm:p-6 space-y-4 sm:space-y-6">
        <div className="flex items-center space-x-3">
          <Shield className="h-8 w-8 text-green-400" />
          <h1 className="text-3xl font-bold">Governance & Rules</h1>
          <div className="bg-green-500/20 text-green-400 px-3 py-1 rounded-full text-sm">247 Rules Active</div>
        </div>

        {/* Metrics Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">Active Rules</p>
                  <p className="text-2xl font-bold text-green-400">198</p>
                  <p className="text-green-400 text-sm">+3 added today</p>
                </div>
                <FileText className="h-8 w-8 text-green-400/70" />
              </div>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">Compliance Score</p>
                  <p className="text-2xl font-bold text-green-400">94.4%</p>
                  <p className="text-green-400 text-sm">Industry leading</p>
                </div>
                <CheckCircle className="h-8 w-8 text-green-400/70" />
              </div>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">Violations</p>
                  <p className="text-2xl font-bold text-red-400">7</p>
                  <p className="text-red-400 text-sm">Under review</p>
                </div>
                <AlertTriangle className="h-8 w-8 text-red-400/70" />
              </div>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">Pending Reviews</p>
                  <p className="text-2xl font-bold text-yellow-400">15</p>
                  <p className="text-yellow-400 text-sm">Requires attention</p>
                </div>
                <XCircle className="h-8 w-8 text-yellow-400/70" />
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Charts Grid */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <Card className="bg-gray-900/50 border-green-500/20">
            <CardHeader>
              <CardTitle className="text-green-400">Rules by Category</CardTitle>
            </CardHeader>
            <CardContent>
              <ChartContainer config={{ active: { color: '#10b981' }, pending: { color: '#f59e0b' }, violations: { color: '#ef4444' } }}>
                <ResponsiveContainer width="100%" height={300}>
                  <BarChart data={ruleCategories}>
                    <XAxis dataKey="category" tick={{ fill: '#10b981', fontSize: 12 }} />
                    <YAxis tick={{ fill: '#10b981', fontSize: 12 }} />
                    <ChartTooltip content={<ChartTooltipContent />} />
                    <Bar dataKey="active" fill="#10b981" />
                    <Bar dataKey="pending" fill="#f59e0b" />
                    <Bar dataKey="violations" fill="#ef4444" />
                  </BarChart>
                </ResponsiveContainer>
              </ChartContainer>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardHeader>
              <CardTitle className="text-green-400">Compliance Status Overview</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {complianceStatus.map((item, index) => (
                  <div key={index} className="flex items-center justify-between p-3 bg-gray-800/50 rounded">
                    <div className="flex items-center space-x-3">
                      <div 
                        className="w-3 h-3 rounded-full" 
                        style={{ backgroundColor: statusColors[item.status] || '#6b7280' }}
                      />
                      <span className="text-green-400">{item.name}</span>
                    </div>
                    <div className="flex items-center space-x-2">
                      <span className="text-green-400 text-sm">{item.score}%</span>
                      <span 
                        className="px-2 py-1 text-xs rounded"
                        style={{ 
                          backgroundColor: `${statusColors[item.status] || '#6b7280'}20`,
                          color: statusColors[item.status] || '#6b7280'
                        }}
                      >
                        {item.status}
                      </span>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Audit Trail */}
        <Card className="bg-gray-900/50 border-green-500/20">
          <CardHeader>
            <CardTitle className="text-green-400">Audit Trail Activity</CardTitle>
          </CardHeader>
          <CardContent>
            <ChartContainer config={{ actions: { color: '#10b981' }, violations: { color: '#ef4444' }, reviews: { color: '#3b82f6' } }}>
              <ResponsiveContainer width="100%" height={300}>
                <LineChart data={auditTrail}>
                  <XAxis dataKey="time" tick={{ fill: '#10b981', fontSize: 12 }} />
                  <YAxis tick={{ fill: '#10b981', fontSize: 12 }} />
                  <ChartTooltip content={<ChartTooltipContent />} />
                  <Line type="monotone" dataKey="actions" stroke="#10b981" strokeWidth={2} />
                  <Line type="monotone" dataKey="violations" stroke="#ef4444" strokeWidth={2} />
                  <Line type="monotone" dataKey="reviews" stroke="#3b82f6" strokeWidth={2} />
                </LineChart>
              </ResponsiveContainer>
            </ChartContainer>
          </CardContent>
        </Card>
      </div>
    </DashboardLayout>
  );
};

export default GovernanceRules;
