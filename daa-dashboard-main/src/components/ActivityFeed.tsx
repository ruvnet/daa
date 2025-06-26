
import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Activity, Bot, DollarSign, Shield, Zap, AlertTriangle, CheckCircle, XCircle } from 'lucide-react';

const ActivityFeed = () => {
  const [activities, setActivities] = useState([
    {
      id: 1,
      type: 'agent_deploy',
      message: 'New Treasury Agent deployed in US-EAST-1',
      timestamp: new Date(Date.now() - 1000 * 60 * 2),
      severity: 'info',
      icon: Bot,
      details: 'Agent ID: TRE-4472, Customer: ACME Corp'
    },
    {
      id: 2,
      type: 'security_alert',
      message: 'Anomalous trading pattern detected',
      timestamp: new Date(Date.now() - 1000 * 60 * 5),
      severity: 'warning',
      icon: Shield,
      details: 'Agent ID: DEF-8821, Threshold exceeded by 15%'
    },
    {
      id: 3,
      type: 'revenue',
      message: 'Monthly revenue milestone reached',
      timestamp: new Date(Date.now() - 1000 * 60 * 8),
      severity: 'success',
      icon: DollarSign,
      details: '$8.95M achieved, 15.3% above target'
    },
    {
      id: 4,
      type: 'ml_training',
      message: 'Federated learning round completed',
      timestamp: new Date(Date.now() - 1000 * 60 * 12),
      severity: 'info',
      icon: Zap,
      details: '247 participants, 96.4% accuracy achieved'
    },
    {
      id: 5,
      type: 'system_error',
      message: 'Node connectivity restored in EU-WEST-1',
      timestamp: new Date(Date.now() - 1000 * 60 * 15),
      severity: 'success',
      icon: CheckCircle,
      details: 'Network partition resolved, 14min downtime'
    },
    {
      id: 6,
      type: 'customer_onboard',
      message: 'New enterprise customer onboarded',
      timestamp: new Date(Date.now() - 1000 * 60 * 20),
      severity: 'info',
      icon: Activity,
      details: 'TechFlow Industries, 50 agent license'
    }
  ]);

  // Simulate real-time activity updates
  useEffect(() => {
    const interval = setInterval(() => {
      const newActivity = {
        id: Date.now(),
        type: 'system_update',
        message: `Agent performance update: ${Math.floor(Math.random() * 1000)} agents processed`,
        timestamp: new Date(),
        severity: 'info' as const,
        icon: Activity,
        details: `Processing efficiency: ${(85 + Math.random() * 10).toFixed(1)}%`
      };
      
      setActivities(prev => [newActivity, ...prev.slice(0, 9)]);
    }, 15000);

    return () => clearInterval(interval);
  }, []);

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'success': return 'text-green-400 border-green-500/30';
      case 'warning': return 'text-yellow-400 border-yellow-500/30';
      case 'error': return 'text-red-400 border-red-500/30';
      default: return 'text-green-400/70 border-green-500/20';
    }
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

  return (
    <Card className="bg-gray-900/50 border-green-500/20">
      <CardHeader>
        <CardTitle className="text-green-400 flex items-center">
          <Activity className="h-5 w-5 mr-2" />
          Real-Time Activity Feed
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="space-y-4 max-h-96 overflow-y-auto scrollbar-thin scrollbar-track-gray-800 scrollbar-thumb-green-500">
          {activities.map((activity) => (
            <div 
              key={activity.id}
              className={`flex items-start space-x-3 p-3 rounded-lg border ${getSeverityColor(activity.severity)} bg-black/30`}
            >
              <div className="flex-shrink-0 mt-1">
                <activity.icon className="h-4 w-4" />
              </div>
              <div className="flex-1 min-w-0">
                <div className="flex items-center justify-between">
                  <p className="text-sm font-medium text-green-400 font-mono">
                    {activity.message}
                  </p>
                  <span className="text-xs text-green-400/50 font-mono">
                    {formatTimestamp(activity.timestamp)}
                  </span>
                </div>
                <p className="text-xs text-green-400/70 mt-1 font-mono">
                  {activity.details}
                </p>
              </div>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
};

export default ActivityFeed;
