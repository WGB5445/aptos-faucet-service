export interface User {
  handle: string;
  role: 'user' | 'privileged' | 'admin';
  max_amount: number;
  max_daily_cap?: number;
  minted_today: number;
  remaining_today?: number;
}

export interface SessionResponse {
  token: string;
  user: User;
}

export interface MintResponse {
  status: 'pending' | 'processing' | 'completed' | 'failed';
  amount: number;
  tx_hash?: string;
  minted_today: number;
  remaining_today?: number;
}

export interface RoleUpdateRequest {
  handle: string;
  channel: 'web' | 'telegram' | 'discord';
  role: 'user' | 'privileged' | 'admin';
}

export interface ApiError {
  message: string;
  status?: number;
}
