
import React, { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { AlertTriangle, Shield, Zap, Server, Eye, X, CheckCircle } from 'lucide-react';

const AlertsPanel = () => {
  const [alerts, setAlerts] = useState([
    {
      id: 1,
      title: 'High CPU Usage',
      description: 'Node US-EAST-1 CPU utilization at 87%',
      severity: 'warning',
      icon: Server,
      timestamp: new Date(Date.now() - 1000 * 60 * 3),
      acknowledged: false
    },
    {
      id: 2,
      title: 'Security Scan Complete',
      description: 'Weekly vulnerability scan found 0 critical issues',
      severity: 'success',
      icon: Shield,
      timestamp: new Date(Date.now() - 1000 * 60 * 15),
      acknowledged: false
    },
    {
      id: 3,
      title: 'ML Training Delayed',
      description: 'Federated learning round 247 delayed due to low participation',
      severity: 'warning',
      icon: Zap,
      timestamp: new Date(Date.now() - 1000 * 60 * 25),
      acknowledged: true
    },
    {
      id: 4,
      title: 'Unusual Network Activity',
      description: 'Increased P2P traffic detected in ASIA-PACIFIC region',
      severity: 'info',
      icon: Eye,
      timestamp: new Date(Date.now() - 1000 * 60 * 45),
      acknowledged: false
    }
  ]);

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'critical': return 'border-red-500 bg-red-500/10 text-red-400';
      case 'warning': return 'border-yellow-500 bg-yellow-500/10 text-yellow-400';
      case 'success': return 'border-green-500 bg-green-500/10 text-green-400';
      default: return 'border-blue-500 bg-blue-500/10 text-blue-400';
    }
  };

  const acknowledgeAlert = (id: number) => {
    setAlerts(prev => prev.map(alert => 
      alert.id === id ? { ...alert, acknowledged: true } : alert
    ));
  };

  const dismissAlert = (id: number) => {
    setAlerts(prev => prev.filter(alert => alert.id !== id));
  };

  const formatTimestamp = (timestamp: Date) => {
    const now = new Date();
    const diff = now.getTime() - timestamp.getTime();
    const minutes = Math.floor(diff / (1000 * 60));
    
    if (minutes < 1) return 'Just now';
    if (minutes < 60) return `${minutes}m ago`;
    const hours = Math.floor(minutes / 60);
    if (hours < 24) return `${hours}h ago`;
    return timestamp.toLocaleDateString();
  };

  const activeAlerts = alerts.filter(alert => !alert.acknowledged);
  const acknowledgedAlerts = alerts.filter(alert => alert.acknowledged);

  return (
    <Card className="bg-gray-900/50 border-green-500/20">
      <CardHeader>
        <CardTitle className="text-green-400 flex items-center justify-between">
          <div className="flex items-center">
            <AlertTriangle className="h-5 w-5 mr-2" />
            System Alerts
          </div>
          <div className="flex items-center space-x-2">
            <span className="bg-red-500/20 text-red-400 text-xs px-2 py-1 rounded-full font-mono">
              {activeAlerts.length} Active
            </span>
          </div>
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="space-y-3 max-h-96 overflow-y-auto scrollbar-thin scrollbar-track-gray-800 scrollbar-thumb-green-500">
          {/* Active Alerts */}
          {activeAlerts.map((alert) => (
            <div 
              key={alert.id}
              className={`border rounded-lg p-3 ${getSeverityColor(alert.severity)}`}
            >
              <div className="flex items-start justify-between">
                <div className="flex items-start space-x-2">
                  <alert.icon className="h-4 w-4 mt-1 flex-shrink-0" />
                  <div>
                    <h4 className="text-sm font-medium font-mono">{alert.title}</h4>
                    <p className="text-xs mt-1 opacity-80 font-mono">{alert.description}</p>
                    <span className="text-xs opacity-60 font-mono">{formatTimestamp(alert.timestamp)}</span>
                  </div>
                </div>
                <div className="flex space-x-1">
                  <button
                    onClick={() => acknowledgeAlert(alert.id)}
                    className="p-1 hover:bg-white/10 rounded transition-colors"
                    title="Acknowledge"
                  >
                    <CheckCircle className="h-3 w-3" />
                  </button>
                  <button
                    onClick={() => dismissAlert(alert.id)}
                    className="p-1 hover:bg-white/10 rounded transition-colors"
                    title="Dismiss"
                  >
                    <X className="h-3 w-3" />
                  </button>
                </div>
              </div>
            </div>
          ))}

          {/* Acknowledged Alerts */}
          {acknowledgedAlerts.length > 0 && (
            <>
              <div className="border-t border-green-500/20 pt-3 mt-4">
                <h5 className="text-xs text-green-400/50 font-mono uppercase tracking-wide mb-2">
                  Acknowledged ({acknowledgedAlerts.length})
                </h5>
              </div>
              {acknowledgedAlerts.map((alert) => (
                <div 
                  key={alert.id}
                  className="border border-gray-600 bg-gray-800/30 rounded-lg p-3 opacity-60"
                >
                  <div className="flex items-start justify-between">
                    <div className="flex items-start space-x-2">
                      <alert.icon className="h-4 w-4 mt-1 flex-shrink-0 text-gray-400" />
                      <div>
                        <h4 className="text-sm font-medium font-mono text-gray-400">{alert.title}</h4>
                        <p className="text-xs mt-1 text-gray-500 font-mono">{alert.description}</p>
                        <span className="text-xs text-gray-600 font-mono">{formatTimestamp(alert.timestamp)}</span>
                      </div>
                    </div>
                    <button
                      onClick={() => dismissAlert(alert.id)}
                      className="p-1 hover:bg-white/10 rounded transition-colors text-gray-400"
                      title="Dismiss"
                    >
                      <X className="h-3 w-3" />
                    </button>
                  </div>
                </div>
              ))}
            </>
          )}

          {alerts.length === 0 && (
            <div className="text-center py-8 text-green-400/50">
              <Shield className="h-8 w-8 mx-auto mb-2" />
              <p className="text-sm font-mono">All systems operational</p>
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  );
};

export default AlertsPanel;
