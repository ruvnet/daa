
import React, { useState } from 'react';
import { Eye, EyeOff, UserPlus, Terminal, Check } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';

interface RegisterFormProps {
  onSwitchToLogin: () => void;
  onRegister: (data: { email: string; password: string; firstName: string; lastName: string; organization: string }) => void;
}

const RegisterForm = ({ onSwitchToLogin, onRegister }: RegisterFormProps) => {
  const [formData, setFormData] = useState({
    firstName: '',
    lastName: '',
    email: '',
    organization: '',
    password: '',
    confirmPassword: ''
  });
  const [showPassword, setShowPassword] = useState(false);
  const [showConfirmPassword, setShowConfirmPassword] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [acceptedTerms, setAcceptedTerms] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (formData.password !== formData.confirmPassword) {
      alert('Passwords do not match');
      return;
    }
    if (!acceptedTerms) {
      alert('Please accept the terms and conditions');
      return;
    }

    setIsLoading(true);
    
    // Simulate API call
    setTimeout(() => {
      onRegister({
        email: formData.email,
        password: formData.password,
        firstName: formData.firstName,
        lastName: formData.lastName,
        organization: formData.organization
      });
      setIsLoading(false);
    }, 1000);
  };

  const updateFormData = (field: string, value: string) => {
    setFormData(prev => ({ ...prev, [field]: value }));
  };

  return (
    <div className="min-h-screen bg-black flex items-center justify-center p-4">
      <div className="w-full max-w-md">
        <div className="text-center mb-8">
          <div className="flex items-center justify-center mb-4">
            <div className="w-12 h-12 bg-green-500 rounded-lg flex items-center justify-center">
              <Terminal className="h-6 w-6 text-black" />
            </div>
          </div>
          <h1 className="text-2xl font-bold text-green-400 font-mono mb-2">DAA Global Command</h1>
          <p className="text-green-400/70 text-sm font-mono">Agent Registration Portal</p>
        </div>

        <div className="bg-gray-900/50 border border-green-500/20 rounded-lg p-6">
          <form onSubmit={handleSubmit} className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label htmlFor="firstName" className="text-green-400 font-mono text-sm">
                  First Name
                </Label>
                <Input
                  id="firstName"
                  type="text"
                  value={formData.firstName}
                  onChange={(e) => updateFormData('firstName', e.target.value)}
                  className="bg-black/50 border-green-500/30 text-green-400 font-mono focus:border-green-500"
                  placeholder="John"
                  required
                />
              </div>
              <div>
                <Label htmlFor="lastName" className="text-green-400 font-mono text-sm">
                  Last Name
                </Label>
                <Input
                  id="lastName"
                  type="text"
                  value={formData.lastName}
                  onChange={(e) => updateFormData('lastName', e.target.value)}
                  className="bg-black/50 border-green-500/30 text-green-400 font-mono focus:border-green-500"
                  placeholder="Doe"
                  required
                />
              </div>
            </div>

            <div>
              <Label htmlFor="email" className="text-green-400 font-mono text-sm">
                Email Address
              </Label>
              <Input
                id="email"
                type="email"
                value={formData.email}
                onChange={(e) => updateFormData('email', e.target.value)}
                className="bg-black/50 border-green-500/30 text-green-400 font-mono focus:border-green-500"
                placeholder="john.doe@company.com"
                required
              />
            </div>

            <div>
              <Label htmlFor="organization" className="text-green-400 font-mono text-sm">
                Organization
              </Label>
              <Input
                id="organization"
                type="text"
                value={formData.organization}
                onChange={(e) => updateFormData('organization', e.target.value)}
                className="bg-black/50 border-green-500/30 text-green-400 font-mono focus:border-green-500"
                placeholder="Acme Corp"
                required
              />
            </div>

            <div>
              <Label htmlFor="password" className="text-green-400 font-mono text-sm">
                Password
              </Label>
              <div className="relative">
                <Input
                  id="password"
                  type={showPassword ? 'text' : 'password'}
                  value={formData.password}
                  onChange={(e) => updateFormData('password', e.target.value)}
                  className="bg-black/50 border-green-500/30 text-green-400 font-mono focus:border-green-500 pr-10"
                  placeholder="••••••••"
                  required
                />
                <button
                  type="button"
                  onClick={() => setShowPassword(!showPassword)}
                  className="absolute right-3 top-1/2 -translate-y-1/2 text-green-400/70 hover:text-green-400"
                >
                  {showPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                </button>
              </div>
            </div>

            <div>
              <Label htmlFor="confirmPassword" className="text-green-400 font-mono text-sm">
                Confirm Password
              </Label>
              <div className="relative">
                <Input
                  id="confirmPassword"
                  type={showConfirmPassword ? 'text' : 'password'}
                  value={formData.confirmPassword}
                  onChange={(e) => updateFormData('confirmPassword', e.target.value)}
                  className="bg-black/50 border-green-500/30 text-green-400 font-mono focus:border-green-500 pr-10"
                  placeholder="••••••••"
                  required
                />
                <button
                  type="button"
                  onClick={() => setShowConfirmPassword(!showConfirmPassword)}
                  className="absolute right-3 top-1/2 -translate-y-1/2 text-green-400/70 hover:text-green-400"
                >
                  {showConfirmPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                </button>
              </div>
            </div>

            <div className="flex items-start space-x-2">
              <button
                type="button"
                onClick={() => setAcceptedTerms(!acceptedTerms)}
                className={`mt-1 w-4 h-4 rounded border-2 flex items-center justify-center ${
                  acceptedTerms 
                    ? 'bg-green-500 border-green-500' 
                    : 'border-green-500/30 bg-black/50'
                }`}
              >
                {acceptedTerms && <Check className="h-3 w-3 text-black" />}
              </button>
              <p className="text-green-400/70 font-mono text-xs">
                I accept the <span className="text-green-400 underline cursor-pointer">Terms of Service</span> and <span className="text-green-400 underline cursor-pointer">Privacy Policy</span>
              </p>
            </div>

            <Button
              type="submit"
              disabled={isLoading}
              className="w-full bg-green-500 hover:bg-green-600 text-black font-mono font-bold"
            >
              {isLoading ? (
                <div className="flex items-center space-x-2">
                  <div className="w-4 h-4 border-2 border-black/30 border-t-black rounded-full animate-spin"></div>
                  <span>CREATING ACCOUNT...</span>
                </div>
              ) : (
                <div className="flex items-center space-x-2">
                  <UserPlus className="h-4 w-4" />
                  <span>CREATE ACCOUNT</span>
                </div>
              )}
            </Button>
          </form>

          <div className="mt-6 text-center">
            <div className="text-green-400/50 font-mono text-xs mb-3">
              ──────── OR ────────
            </div>
            <button
              onClick={onSwitchToLogin}
              className="text-green-400 hover:text-green-300 font-mono text-sm"
            >
              Already have an account? Sign In
            </button>
          </div>
        </div>

        <div className="mt-6 text-center">
          <p className="text-green-400/50 font-mono text-xs">
            © 2025 DAA Technologies • Quantum-Secured Platform
          </p>
        </div>
      </div>
    </div>
  );
};

export default RegisterForm;
