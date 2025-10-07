import { User, SessionResponse, MintRequest, MintResponse, RoleUpdateRequest } from '../types';

const API_BASE_URL = (import.meta as any).env?.VITE_API_BASE_URL || '/api';

// Helper function to make API requests
async function apiRequest<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
  const token = localStorage.getItem('auth_token');
  
  const config: RequestInit = {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...(token && { Authorization: `Bearer ${token}` }),
      ...options.headers,
    },
  };

  const response = await fetch(`${API_BASE_URL}${endpoint}`, config);
  
  if (response.status === 401) {
    localStorage.removeItem('auth_token');
    window.location.href = '/';
    throw new Error('Unauthorized');
  }
  
  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`);
  }
  
  return response.json();
}

export const authApi = {
  async createSession(idToken: string): Promise<SessionResponse> {
    return apiRequest<SessionResponse>('/api/session', {
      method: 'POST',
      body: JSON.stringify({ id_token: idToken }),
    });
  },

  async getCurrentUser(): Promise<User> {
    return apiRequest<User>('/api/me');
  },
};

export const faucetApi = {
  async mintTokens(amount?: number, walletAddress?: string): Promise<MintResponse> {
    const request: MintRequest = { 
      amount,
      wallet_address: walletAddress 
    };
    return apiRequest<MintResponse>('/api/mint', {
      method: 'POST',
      body: JSON.stringify(request),
    });
  },
};

export const adminApi = {
  async updateRole(request: RoleUpdateRequest): Promise<User> {
    return apiRequest<User>('/api/admin/role', {
      method: 'POST',
      body: JSON.stringify(request),
    });
  },
};
