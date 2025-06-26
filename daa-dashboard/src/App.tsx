import { Toaster } from "@/components/ui/toaster";
import { Toaster as Sonner } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { useState } from "react";
import Index from "./pages/Index";
import Auth from "./pages/Auth";
import NotFound from "./pages/NotFound";
import AgentManagement from "./pages/AgentManagement";
import EconomicManagement from "./pages/EconomicManagement";
import NetworkOperations from "./pages/NetworkOperations";
import GovernanceRules from "./pages/GovernanceRules";
import AIMLOperations from "./pages/AIMLOperations";
import CustomerManagement from "./pages/CustomerManagement";
import AnalyticsReporting from "./pages/AnalyticsReporting";
import SystemAdministration from "./pages/SystemAdministration";
import SecurityCompliance from "./pages/SecurityCompliance";

const queryClient = new QueryClient();

const App = () => {
  const [isAuthenticated, setIsAuthenticated] = useState(false);

  if (!isAuthenticated) {
    return (
      <QueryClientProvider client={queryClient}>
        <TooltipProvider>
          <Toaster />
          <Sonner />
          <Auth onAuthenticated={() => setIsAuthenticated(true)} />
        </TooltipProvider>
      </QueryClientProvider>
    );
  }

  return (
    <QueryClientProvider client={queryClient}>
      <TooltipProvider>
        <Toaster />
        <Sonner />
        <BrowserRouter>
          <Routes>
            <Route path="/" element={<Index />} />
            <Route path="/agent-management" element={<AgentManagement />} />
            <Route path="/economic-management" element={<EconomicManagement />} />
            <Route path="/network-operations" element={<NetworkOperations />} />
            <Route path="/governance-rules" element={<GovernanceRules />} />
            <Route path="/ai-ml-operations" element={<AIMLOperations />} />
            <Route path="/customer-management" element={<CustomerManagement />} />
            <Route path="/analytics-reporting" element={<AnalyticsReporting />} />
            <Route path="/system-administration" element={<SystemAdministration />} />
            <Route path="/security-compliance" element={<SecurityCompliance />} />
            {/* ADD ALL CUSTOM ROUTES ABOVE THE CATCH-ALL "*" ROUTE */}
            <Route path="*" element={<NotFound />} />
          </Routes>
        </BrowserRouter>
      </TooltipProvider>
    </QueryClientProvider>
  );
};

export default App;
