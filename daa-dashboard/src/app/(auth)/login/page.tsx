'use client';

import { LoginForm } from '@/components/auth/LoginForm';
import { Card } from '@/components/ui/card';

export default function LoginPage() {
  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900">
      <Card className="w-full max-w-md p-8">
        <div className="mb-8 text-center">
          <h1 className="text-2xl font-bold">Welcome to DAA Dashboard</h1>
          <p className="text-muted-foreground mt-2">
            Sign in to manage your autonomous agents
          </p>
        </div>
        <LoginForm />
      </Card>
    </div>
  );
}