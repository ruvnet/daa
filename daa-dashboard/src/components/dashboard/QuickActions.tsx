'use client';

import { Button } from '@/components/ui/button';
import { 
  PlusIcon, 
  UserPlusIcon, 
  FileTextIcon, 
  DownloadIcon, 
  AlertCircleIcon, 
  CalendarIcon 
} from 'lucide-react';

export function QuickActions() {
  const actions = [
    {
      label: 'Deploy Agent Fleet',
      icon: <PlusIcon className="h-4 w-4" />,
      onClick: () => console.log('Deploy agent fleet'),
    },
    {
      label: 'Create Customer',
      icon: <UserPlusIcon className="h-4 w-4" />,
      onClick: () => console.log('Create customer'),
    },
    {
      label: 'Generate Report',
      icon: <FileTextIcon className="h-4 w-4" />,
      onClick: () => console.log('Generate report'),
    },
    {
      label: 'System Backup',
      icon: <DownloadIcon className="h-4 w-4" />,
      onClick: () => console.log('System backup'),
    },
    {
      label: 'Open Incident',
      icon: <AlertCircleIcon className="h-4 w-4" />,
      onClick: () => console.log('Open incident'),
    },
    {
      label: 'Schedule Maintenance',
      icon: <CalendarIcon className="h-4 w-4" />,
      onClick: () => console.log('Schedule maintenance'),
    },
  ];

  return (
    <div className="space-y-2">
      {actions.map((action, index) => (
        <Button
          key={index}
          variant="outline"
          className="w-full justify-start"
          onClick={action.onClick}
        >
          {action.icon}
          <span className="ml-2">{action.label}</span>
        </Button>
      ))}
    </div>
  );
}