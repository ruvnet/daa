import React from 'react';
import { Eye, Shield, AlertTriangle, Lock, FileCheck, UserCheck } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { ChartContainer, ChartTooltip, ChartTooltipContent } from '@/components/ui/chart';
import { LineChart, Line, XAxis, YAxis, ResponsiveContainer, AreaChart, Area, BarChart, Bar, PieChart, Pie, Cell } from 'recharts';
import DashboardLayout from '@/components/DashboardLayout';

const SecurityCompliance = () => {
  const threatDetection = [
    { time: '00:00', threats: 12, blocked: 11, investigated: 1 },
    { time: '04:00', threats: 8, blocked: 8, investigated: 0 },
    { time: '08:00', threats: 23, blocked: 22, investigated: 1 },
    { time: '12:00', threats: 45, blocked: 43, investigated: 2 },
    { time: '16:00', threats: 34, blocked: 33, investigated: 1 },
    { time: '20:00', threats: 19, blocked: 18, investigated: 1 }
  ];

  const complianceStatus = [
    { framework: 'GDPR', score: 98, status: 'Compliant' },
    { framework: 'SOX', score: 95, status: 'Compliant' },
    { framework: 'PCI DSS', score: 99, status: 'Compliant' },
    { framework: 'ISO 27001', score: 88, status: 'Review Required' },
    { framework: 'HIPAA', score: 92, status: 'Compliant' }
  ];

  const securityIncidents = [
    { severity: 'Critical', count: 2, color: '#ef4444' },
    { severity: 'High', count: 8, color: '#f59e0b' },
    { severity: 'Medium', count: 23, color: '#3b82f6' },
    { severity: 'Low', count: 45, color: '#10b981' },
    { severity: 'Info', count: 67, color: '#6b7280' }
  ];

  const accessLogs = [
    { time: '00:00', logins: 234, failed: 12, suspicious: 2 },
    { time: '04:00', logins: 189, failed: 8, suspicious: 1 },
    { time: '08:00', logins: 567, failed: 23, suspicious: 5 },
    { time: '12:00', logins: 634, failed: 34, suspicious: 7 },
    { time: '16:00', logins: 489, failed: 19, suspicious: 3 },
    { time: '20:00', logins: 356, failed: 15, suspicious: 2 }
  ];

  const vulnerabilityScans = [
    { date: 'Week 1', critical: 2, high: 8, medium: 15, low: 23 },
    { date: 'Week 2', critical: 1, high: 6, medium: 12, low: 19 },
    { date: 'Week 3', critical: 0, high: 4, medium: 9, low: 16 },
    { date: 'Week 4', critical: 0, high: 2, medium: 7, low: 14 }
  ];

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'Compliant': return '#10b981';
      case 'Review Required': return '#f59e0b';
      case 'Non-Compliant': return '#ef4444';
      default: return '#6b7280';
    }
  };

  return (
    <DashboardLayout>
      <div className="p-3 sm:p-6 space-y-4 sm:space-y-6">
        <div className="flex items-center space-x-3">
          <Eye className="h-8 w-8 text-green-400" />
          <h1 className="text-3xl font-bold">Security & Compliance</h1>
          <div className="bg-green-500/20 text-green-400 px-3 py-1 rounded-full text-sm">3 Active Alerts</div>
        </div>

        {/* Metrics Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">Threats Blocked</p>
                  <p className="text-2xl font-bold text-green-400">135</p>
                  <p className="text-green-400 text-sm">99.2% success rate</p>
                </div>
                <Eye className="h-8 w-8 text-green-400/70" />
              </div>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">Security Score</p>
                  <p className="text-2xl font-bold text-green-400">94.4</p>
                  <p className="text-green-400 text-sm">Excellent rating</p>
                </div>
                <Lock className="h-8 w-8 text-green-400/70" />
              </div>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">Compliance</p>
                  <p className="text-2xl font-bold text-green-400">96%</p>
                  <p className="text-green-400 text-sm">5 frameworks</p>
                </div>
                <FileCheck className="h-8 w-8 text-green-400/70" />
              </div>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-green-400/70 text-sm">Active Incidents</p>
                  <p className="text-2xl font-bold text-yellow-400">6</p>
                  <p className="text-yellow-400 text-sm">Under investigation</p>
                </div>
                <AlertTriangle className="h-8 w-8 text-yellow-400/70" />
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Charts Grid */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <Card className="bg-gray-900/50 border-green-500/20">
            <CardHeader>
              <CardTitle className="text-green-400">Threat Detection & Response</CardTitle>
            </CardHeader>
            <CardContent>
              <ChartContainer config={{ threats: { color: '#ef4444' }, blocked: { color: '#10b981' }, investigated: { color: '#f59e0b' } }}>
                <ResponsiveContainer width="100%" height={300}>
                  <AreaChart data={threatDetection}>
                    <XAxis dataKey="time" tick={{ fill: '#10b981', fontSize: 12 }} />
                    <YAxis tick={{ fill: '#10b981', fontSize: 12 }} />
                    <ChartTooltip content={<ChartTooltipContent />} />
                    <Area type="monotone" dataKey="threats" stackId="1" stroke="#ef4444" fill="#ef4444" fillOpacity={0.3} />
                    <Area type="monotone" dataKey="blocked" stackId="2" stroke="#10b981" fill="#10b981" fillOpacity={0.6} />
                    <Area type="monotone" dataKey="investigated" stackId="3" stroke="#f59e0b" fill="#f59e0b" fillOpacity={0.4} />
                  </AreaChart>
                </ResponsiveContainer>
              </ChartContainer>
            </CardContent>
          </Card>

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardHeader>
              <CardTitle className="text-green-400">Security Incident Distribution</CardTitle>
            </CardHeader>
            <CardContent>
              <ChartContainer config={{}}>
                <ResponsiveContainer width="100%" height={300}>
                  <PieChart>
                    <Pie
                      data={securityIncidents}
                      cx="50%"
                      cy="50%"
                      outerRadius={100}
                      dataKey="count"
                      label={({ severity, count }) => `${severity}: ${count}`}
                    >
                      {securityIncidents.map((entry, index) => (
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

        {/* Compliance and Access Logs */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <Card className="bg-gray-900/50 border-green-500/20">
            <CardHeader>
              <CardTitle className="text-green-400">Compliance Status</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {complianceStatus.map((item, index) => (
                  <div key={index} className="flex items-center justify-between p-3 bg-gray-800/50 rounded">
                    <div className="flex items-center space-x-3">
                      <div 
                        className="w-3 h-3 rounded-full" 
                        style={{ backgroundColor: getStatusColor(item.status) }}
                      />
                      <span className="text-green-400">{item.framework}</span>
                    </div>
                    <div className="flex items-center space-x-4 text-sm">
                      <span className="text-green-400 font-bold">{item.score}%</span>
                      <span 
                        className="px-2 py-1 text-xs rounded"
                        style={{ 
                          backgroundColor: `${getStatusColor(item.status)}20`,
                          color: getStatusColor(item.status)
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

          <Card className="bg-gray-900/50 border-green-500/20">
            <CardHeader>
              <CardTitle className="text-green-400">Access Logs & Authentication</CardTitle>
            </CardHeader>
            <CardContent>
              <ChartContainer config={{ logins: { color: '#10b981' }, failed: { color: '#ef4444' }, suspicious: { color: '#f59e0b' } }}>
                <ResponsiveContainer width="100%" height={300}>
                  <LineChart data={accessLogs}>
                    <XAxis dataKey="time" tick={{ fill: '#10b981', fontSize: 12 }} />
                    <YAxis tick={{ fill: '#10b981', fontSize: 12 }} />
                    <ChartTooltip content={<ChartTooltipContent />} />
                    <Line type="monotone" dataKey="logins" stroke="#10b981" strokeWidth={2} />
                    <Line type="monotone" dataKey="failed" stroke="#ef4444" strokeWidth={2} />
                    <Line type="monotone" dataKey="suspicious" stroke="#f59e0b" strokeWidth={2} />
                  </LineChart>
                </ResponsiveContainer>
              </ChartContainer>
            </CardContent>
          </Card>
        </div>

        {/* Vulnerability Scans */}
        <Card className="bg-gray-900/50 border-green-500/20">
          <CardHeader>
            <CardTitle className="text-green-400">Vulnerability Scan Results</CardTitle>
          </CardHeader>
          <CardContent>
            <ChartContainer config={{ critical: { color: '#ef4444' }, high: { color: '#f59e0b' }, medium: { color: '#3b82f6' }, low: { color: '#10b981' } }}>
              <ResponsiveContainer width="100%" height={300}>
                <BarChart data={vulnerabilityScans}>
                  <XAxis dataKey="date" tick={{ fill: '#10b981', fontSize: 12 }} />
                  <YAxis tick={{ fill: '#10b981', fontSize: 12 }} />
                  <ChartTooltip content={<ChartTooltipContent />} />
                  <Bar dataKey="critical" stackId="a" fill="#ef4444" />
                  <Bar dataKey="high" stackId="a" fill="#f59e0b" />
                  <Bar dataKey="medium" stackId="a" fill="#3b82f6" />
                  <Bar dataKey="low" stackId="a" fill="#10b981" />
                </BarChart>
              </ResponsiveContainer>
            </ChartContainer>
          </CardContent>
        </Card>
      </div>
    </DashboardLayout>
  );
};

export default SecurityCompliance;
