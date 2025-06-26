import React, { useState, useEffect } from 'react';
import { 
  Activity, 
  Globe, 
  Shield, 
  Zap, 
  TrendingUp, 
  Users, 
  Server, 
  DollarSign,
  Bot,
  Network,
  Eye,
  Terminal,
  Settings,
  Bell,
  Search,
  Menu,
  X,
  LogOut
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Link, useLocation } from 'react-router-dom';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';

interface DashboardLayoutProps {
  children: React.ReactNode;
}

const DashboardLayout = ({ children }: DashboardLayoutProps) => {
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [currentTime, setCurrentTime] = useState(new Date());
  const [searchOpen, setSearchOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [notificationsOpen, setNotificationsOpen] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const location = useLocation();

  useEffect(() => {
    const timer = setInterval(() => {
      setCurrentTime(new Date());
    }, 5000);

    return () => clearInterval(timer);
  }, []);

  const navigationItems = [
    { name: 'Dashboard', icon: Activity, active: location.pathname === '/', path: '/' },
    { name: 'Agent Management', icon: Bot, badge: '1.2K', path: '/agent-management' },
    { name: 'Economic Management', icon: DollarSign, badge: '+15%', path: '/economic-management' },
    { name: 'Network Operations', icon: Network, path: '/network-operations' },
    { name: 'Governance & Rules', icon: Shield, path: '/governance-rules' },
    { name: 'AI & ML Operations', icon: Zap, path: '/ai-ml-operations' },
    { name: 'Customer Management', icon: Users, badge: '2.8K', path: '/customer-management' },
    { name: 'Analytics & Reporting', icon: TrendingUp, path: '/analytics-reporting' },
    { name: 'System Administration', icon: Server, path: '/system-administration' },
    { name: 'Security & Compliance', icon: Eye, badge: '3', path: '/security-compliance' }
  ];

  const notifications = [
    { id: 1, title: 'High CPU Usage', message: 'Node US-EAST-1 at 87%', time: '3m ago', urgent: true },
    { id: 2, title: 'Security Scan Complete', message: '0 critical issues found', time: '15m ago', urgent: false },
    { id: 3, title: 'ML Training Delayed', message: 'Round 247 delayed', time: '25m ago', urgent: false },
    { id: 4, title: 'Network Activity', message: 'Increased P2P traffic', time: '45m ago', urgent: false },
    { id: 5, title: 'Agent Deployment', message: '12 new agents online', time: '1h ago', urgent: false }
  ];

  const handleNavClick = () => {
    window.scrollTo({ top: 0, behavior: 'smooth' });
    setSidebarOpen(false);
  };

  const handleLogout = () => {
    window.location.reload();
  };

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    console.log('Searching for:', searchQuery);
    // Here you would implement actual search functionality
    setSearchOpen(false);
    setSearchQuery('');
  };

  const markNotificationAsRead = (id: number) => {
    console.log('Marking notification as read:', id);
    // Here you would implement notification marking functionality
  };

  return (
    <div className="min-h-screen bg-black text-green-400 font-mono">
      {/* Header */}
      <header className="sticky top-0 z-50 bg-black/90 backdrop-blur-sm border-b border-green-500/20">
        <div className="flex items-center justify-between px-4 sm:px-6 py-3 sm:py-4">
          <div className="flex items-center space-x-3 sm:space-x-4">
            <button
              onClick={() => setSidebarOpen(!sidebarOpen)}
              className="lg:hidden p-2 hover:bg-green-500/10 rounded-lg transition-colors"
            >
              {sidebarOpen ? <X className="h-5 w-5 sm:h-6 sm:w-6" /> : <Menu className="h-5 w-5 sm:h-6 sm:w-6" />}
            </button>
            <div className="flex items-center space-x-2 sm:space-x-3">
              <div className="w-6 h-6 sm:w-8 sm:h-8 bg-green-500 rounded-lg flex items-center justify-center">
                <Terminal className="h-3 w-3 sm:h-5 sm:w-5 text-black" />
              </div>
              <div className="hidden sm:block">
                <h1 className="text-lg sm:text-xl font-bold text-green-400">DAA Global Command</h1>
                <p className="text-xs text-green-400/60">Decentralized Autonomous Agents</p>
              </div>
              <div className="sm:hidden">
                <h1 className="text-sm font-bold text-green-400">DAA Command</h1>
              </div>
            </div>
          </div>
          
          <div className="flex items-center space-x-2 sm:space-x-4">
            <div className="hidden md:block text-xs sm:text-sm text-green-400/70">
              <span>UTC: {currentTime.toISOString().slice(0, 19)}Z</span>
            </div>
            <div className="flex items-center space-x-1 sm:space-x-2">
              {/* Search Icon */}
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setSearchOpen(true)}
                className="p-1 sm:p-2 text-green-400/70 hover:text-green-400 hover:bg-green-500/10"
              >
                <Search className="h-4 w-4 sm:h-5 sm:w-5" />
                <span className="sr-only">Search</span>
              </Button>

              {/* Notifications Bell */}
              <Popover open={notificationsOpen} onOpenChange={setNotificationsOpen}>
                <PopoverTrigger asChild>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="p-1 sm:p-2 text-green-400/70 hover:text-green-400 hover:bg-green-500/10 relative"
                  >
                    <Bell className="h-4 w-4 sm:h-5 sm:w-5" />
                    <div className="absolute -top-1 -right-1 w-2 h-2 bg-red-500 rounded-full animate-pulse"></div>
                    <span className="sr-only">Notifications ({notifications.filter(n => n.urgent).length} urgent)</span>
                  </Button>
                </PopoverTrigger>
                <PopoverContent className="w-80 bg-gray-900 border-green-500/20 text-green-400">
                  <div className="space-y-3">
                    <div className="flex items-center justify-between">
                      <h3 className="font-mono text-sm font-bold">System Notifications</h3>
                      <span className="text-xs bg-red-500/20 text-red-400 px-2 py-1 rounded-full">
                        {notifications.filter(n => n.urgent).length} Urgent
                      </span>
                    </div>
                    <div className="max-h-80 overflow-y-auto space-y-2">
                      {notifications.slice(0, 5).map((notification) => (
                        <div
                          key={notification.id}
                          className={`p-3 rounded border cursor-pointer hover:bg-green-500/5 ${
                            notification.urgent 
                              ? 'border-red-500/30 bg-red-500/5' 
                              : 'border-green-500/20'
                          }`}
                          onClick={() => markNotificationAsRead(notification.id)}
                        >
                          <div className="flex justify-between items-start">
                            <div>
                              <h4 className="text-xs font-mono font-semibold">{notification.title}</h4>
                              <p className="text-xs text-green-400/70 mt-1">{notification.message}</p>
                            </div>
                            <span className="text-xs text-green-400/50">{notification.time}</span>
                          </div>
                        </div>
                      ))}
                    </div>
                    <div className="pt-2 border-t border-green-500/20">
                      <Button variant="ghost" size="sm" className="w-full text-green-400/70 hover:text-green-400 text-xs">
                        View All Notifications
                      </Button>
                    </div>
                  </div>
                </PopoverContent>
              </Popover>

              {/* Settings */}
              <Popover open={settingsOpen} onOpenChange={setSettingsOpen}>
                <PopoverTrigger asChild>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="p-1 sm:p-2 text-green-400/70 hover:text-green-400 hover:bg-green-500/10"
                  >
                    <Settings className="h-4 w-4 sm:h-5 sm:w-5" />
                    <span className="sr-only">Settings</span>
                  </Button>
                </PopoverTrigger>
                <PopoverContent className="w-56 bg-gray-900 border-green-500/20 text-green-400">
                  <div className="space-y-2">
                    <h3 className="font-mono text-sm font-bold mb-3">Quick Settings</h3>
                    <div className="space-y-1">
                      <Button variant="ghost" size="sm" className="w-full justify-start text-green-400/70 hover:text-green-400 text-xs">
                        <Users className="h-3 w-3 mr-2" />
                        Account Settings
                      </Button>
                      <Button variant="ghost" size="sm" className="w-full justify-start text-green-400/70 hover:text-green-400 text-xs">
                        <Shield className="h-3 w-3 mr-2" />
                        Security Settings
                      </Button>
                      <Button variant="ghost" size="sm" className="w-full justify-start text-green-400/70 hover:text-green-400 text-xs">
                        <Bell className="h-3 w-3 mr-2" />
                        Notification Preferences
                      </Button>
                      <Button variant="ghost" size="sm" className="w-full justify-start text-green-400/70 hover:text-green-400 text-xs">
                        <Terminal className="h-3 w-3 mr-2" />
                        System Preferences
                      </Button>
                      <div className="border-t border-green-500/20 pt-2 mt-2">
                        <Button variant="ghost" size="sm" className="w-full justify-start text-green-400/70 hover:text-green-400 text-xs">
                          <Eye className="h-3 w-3 mr-2" />
                          Theme: Hacker Dark
                        </Button>
                      </div>
                    </div>
                  </div>
                </PopoverContent>
              </Popover>
            </div>
            <div className="flex items-center space-x-2 bg-green-500/10 px-2 sm:px-3 py-1 sm:py-2 rounded-lg">
              <div className="w-2 h-2 bg-green-400 rounded-full animate-pulse"></div>
              <span className="text-xs sm:text-sm">ADMIN</span>
            </div>
            <Button
              onClick={handleLogout}
              variant="ghost"
              size="sm"
              className="text-green-400/70 hover:text-green-400 p-1 sm:p-2"
            >
              <LogOut className="h-4 w-4" />
            </Button>
          </div>
        </div>
      </header>

      {/* Search Dialog */}
      <Dialog open={searchOpen} onOpenChange={setSearchOpen}>
        <DialogContent className="bg-gray-900 border-green-500/20 text-green-400">
          <DialogHeader>
            <DialogTitle className="text-green-400 font-mono">Global Search</DialogTitle>
          </DialogHeader>
          <form onSubmit={handleSearch} className="space-y-4">
            <Input
              type="text"
              placeholder="Search agents, customers, transactions..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="bg-black/50 border-green-500/30 text-green-400 font-mono focus:border-green-500"
              autoFocus
            />
            <div className="flex justify-end space-x-2">
              <Button 
                type="button" 
                variant="ghost" 
                onClick={() => setSearchOpen(false)}
                className="text-green-400/70 hover:text-green-400"
              >
                Cancel
              </Button>
              <Button 
                type="submit" 
                className="bg-green-500 hover:bg-green-600 text-black font-mono"
              >
                Search
              </Button>
            </div>
          </form>
        </DialogContent>
      </Dialog>

      <div className="flex">
        {/* Sidebar */}
        <aside className={`fixed lg:relative inset-y-0 left-0 z-40 w-56 sm:w-64 transform ${sidebarOpen ? 'translate-x-0' : '-translate-x-full'} lg:translate-x-0 transition-transform duration-300 ease-in-out bg-gray-900/50 border-r border-green-500/20`}>
          <div className="flex flex-col h-full pt-16 lg:pt-6">
            <nav className="flex-1 px-3 sm:px-4 pb-4 space-y-1 sm:space-y-2 overflow-y-auto">
              {navigationItems.map((item) => (
                <Link
                  key={item.name}
                  to={item.path}
                  onClick={handleNavClick}
                  className={`flex items-center justify-between px-3 sm:px-4 py-2 sm:py-3 text-xs sm:text-sm rounded-lg transition-all duration-200 ${
                    item.active 
                      ? 'bg-green-500/20 text-green-400 border border-green-500/30' 
                      : 'text-green-400/70 hover:bg-green-500/10 hover:text-green-400'
                  }`}
                >
                  <div className="flex items-center min-w-0">
                    <item.icon className="h-4 w-4 sm:h-5 sm:w-5 mr-2 sm:mr-3 flex-shrink-0" />
                    <span className="truncate">{item.name}</span>
                  </div>
                  {item.badge && (
                    <span className="bg-green-500/20 text-green-400 text-xs px-2 py-1 rounded-full flex-shrink-0">
                      {item.badge}
                    </span>
                  )}
                </Link>
              ))}
            </nav>
          </div>
        </aside>

        {/* Main Content */}
        <main className="flex-1 lg:ml-0">
          {children}
        </main>
      </div>

      {/* Footer */}
      <footer className="sticky bottom-0 bg-black/90 backdrop-blur-sm border-t border-green-500/20 px-3 sm:px-6 py-2 sm:py-3">
        <div className="flex flex-col sm:flex-row items-center justify-between text-xs sm:text-sm text-green-400/70 space-y-1 sm:space-y-0">
          <div className="flex items-center space-x-3 sm:space-x-6">
            <span>Load: 23.4%</span>
            <span className="hidden sm:inline">Memory: 67.8GB/128GB</span>
            <span>Network: 2.3GB/s</span>
          </div>
          <div className="flex items-center space-x-4">
            <span className="hidden sm:inline">DAA Global Command v2.1.0</span>
            <div className="flex items-center space-x-1">
              <div className="w-2 h-2 bg-green-400 rounded-full animate-pulse"></div>
              <span>OPERATIONAL</span>
            </div>
          </div>
        </div>
      </footer>

      {/* Mobile Sidebar Overlay */}
      {sidebarOpen && (
        <div 
          className="fixed inset-0 z-30 bg-black/50 lg:hidden" 
          onClick={() => setSidebarOpen(false)}
        />
      )}
    </div>
  );
};

export default DashboardLayout;
