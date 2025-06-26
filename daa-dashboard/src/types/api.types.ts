// API Response Types
export interface ApiResponse<T> {
  data: T;
  error?: ApiError;
  meta?: ApiMeta;
}

export interface ApiError {
  code: string;
  message: string;
  details?: Record<string, any>;
}

export interface ApiMeta {
  page?: number;
  pageSize?: number;
  total?: number;
  hasMore?: boolean;
}

// Authentication Types
export interface LoginCredentials {
  email: string;
  password: string;
  twoFactorCode?: string;
}

export interface LoginResponse {
  user: User;
  token: string;
  refreshToken: string;
  expiresIn: number;
}

export interface User {
  id: string;
  email: string;
  name: string;
  role: UserRole;
  permissions: string[];
  createdAt: string;
  lastLogin?: string;
  avatar?: string;
  organization?: Organization;
}

export type UserRole = 
  | 'super_admin'
  | 'business_admin'
  | 'operations_manager'
  | 'developer'
  | 'analyst'
  | 'customer';

export interface Organization {
  id: string;
  name: string;
  tier: 'enterprise' | 'professional' | 'standard' | 'basic';
}

// Query Options
export interface QueryOptions {
  page?: number;
  pageSize?: number;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
  filters?: Record<string, any>;
}

// WebSocket Events
export interface WebSocketEvent<T = any> {
  type: string;
  payload: T;
  timestamp: string;
  id: string;
}