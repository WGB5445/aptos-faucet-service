import axios from 'axios';
import { User, SessionResponse, MintResponse, RoleUpdateRequest } from '../types';

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
  console.log('API request:', config.method?.toUpperCase(), config.url, 'with token:', !!token);
  return config;
});

// Response interceptor to handle errors
api.interceptors.response.use(
  (response: any) => {
    console.log('API response:', response.status, response.config.url);
    return response;
  },
  (error: any) => {
    console.error('API error:', error.response?.status, error.config?.url, error.message);
    if (error.response?.status === 401) {
      console.log('API: 401 error detected, clearing auth_token from localStorage');
      localStorage.removeItem('auth_token');
      console.log('API: Token cleared, redirecting to home page');
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
  async mintTokens(amount?: number): Promise<MintResponse> {
    const response = await api.post('/api/mint', { amount });
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
