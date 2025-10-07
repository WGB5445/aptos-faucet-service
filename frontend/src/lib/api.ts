import axios from 'axios';
import { User, SessionResponse, MintRequest, MintResponse, RoleUpdateRequest } from '../types';

const API_BASE_URL = (import.meta as any).env?.VITE_API_BASE_URL || '/api';

const api = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Request interceptor to add auth token
api.interceptors.request.use((config: any) => {
  const token = localStorage.getItem('auth_token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// Response interceptor to handle errors
api.interceptors.response.use(
  (response: any) => {
    return response;
  },
  (error: any) => {
    if (error.response?.status === 401) {
      localStorage.removeItem('auth_token');
      // 重定向到首页，让LandingPage处理登录状态
      window.location.href = '/';
    }
    return Promise.reject(error);
  }
);

export const authApi = {
  async createSession(idToken: string): Promise<SessionResponse> {
    const response = await api.post('/api/session', { id_token: idToken });
    return response.data;
  },

  async getCurrentUser(): Promise<User> {
    const response = await api.get('/api/me');
    return response.data;
  },
};

export const faucetApi = {
  async mintTokens(amount?: number, walletAddress?: string): Promise<MintResponse> {
    const request: MintRequest = { 
      amount,
      wallet_address: walletAddress 
    };
    const response = await api.post('/api/mint', request);
    return response.data;
  },
};

export const adminApi = {
  async updateRole(request: RoleUpdateRequest): Promise<User> {
    const response = await api.post('/api/admin/role', request);
    return response.data;
  },
};

export default api;
