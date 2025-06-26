import React from 'react';
import { Card, CardContent } from '@/components/ui/card';
import { LucideIcon } from 'lucide-react';
import { cn } from '@/lib/utils';

interface MetricCardProps {
  title: string;
  value: string | number;
  change?: string;
  trend?: 'up' | 'down' | 'neutral';
  icon: LucideIcon;
  suffix?: string;
  prefix?: string;
  className?: string;
  iconClassName?: string;
  valueClassName?: string;
}

const MetricCard: React.FC<MetricCardProps> = ({
  title,
  value,
  change,
  trend = 'neutral',
  icon: Icon,
  suffix = '',
  prefix = '',
  className,
  iconClassName,
  valueClassName,
}) => {
  const getTrendColor = () => {
    switch (trend) {
      case 'up':
        return 'text-green-400';
      case 'down':
        return 'text-red-400';
      default:
        return 'text-gray-400';
    }
  };

  return (
    <Card className={cn(
      'bg-gray-900/50 border-green-500/20 hover:border-green-500/40 transition-all duration-300',
      className
    )}>
      <CardContent className="p-4 sm:p-6">
        <div className="flex items-center justify-between">
          <div className="flex-1 min-w-0">
            <p className="text-green-400/70 text-xs sm:text-sm font-mono uppercase tracking-wide truncate">
              {title}
            </p>
            <p className={cn(
              'text-lg sm:text-2xl font-bold text-green-400 font-mono mt-1 truncate',
              valueClassName
            )}>
              {prefix}{typeof value === 'number' ? value.toLocaleString() : value}{suffix}
            </p>
            {change && (
              <div className={cn(
                'flex items-center mt-1 sm:mt-2 text-xs sm:text-sm font-mono',
                getTrendColor()
              )}>
                <span className="truncate">{change}</span>
              </div>
            )}
          </div>
          <div className={cn(
            'bg-green-500/10 p-2 sm:p-3 rounded-lg ml-2 flex-shrink-0',
            iconClassName
          )}>
            <Icon className="h-5 w-5 sm:h-8 sm:w-8 text-green-400" />
          </div>
        </div>
      </CardContent>
    </Card>
  );
};

export default MetricCard;