import { authApi } from './api';

// Google OAuth configuration
const GOOGLE_CLIENT_ID = (import.meta as any).env?.VITE_GOOGLE_CLIENT_ID || '';

export interface GoogleUser {
  id: string;
  email: string;
  name: string;
  picture: string;
}

// 声明全局的 Google Identity Services 类型
declare global {
  interface Window {
    google: any;
  }
}

class GoogleAuthService {
  private isInitialized = false;

  async initialize(): Promise<void> {
    if (this.isInitialized) return;

    return new Promise((resolve, reject) => {
      // 加载新的 Google Identity Services 库
      const script = document.createElement('script');
      script.src = 'https://accounts.google.com/gsi/client';
      script.onload = () => {
        if (window.google) {
          window.google.accounts.id.initialize({
            client_id: GOOGLE_CLIENT_ID,
            callback: this.handleCredentialResponse.bind(this),
            auto_select: false,
            cancel_on_tap_outside: false,
          });
          this.isInitialized = true;
          resolve();
        } else {
          reject(new Error('Failed to load Google Identity Services'));
        }
      };
      script.onerror = reject;
      document.head.appendChild(script);
    });
  }

  private async handleCredentialResponse(response: any): Promise<void> {
    try {
      // 解析 JWT token
      const payload = this.parseJwt(response.credential);
      
      const googleUser: GoogleUser = {
        id: payload.sub,
        email: payload.email,
        name: payload.name,
        picture: payload.picture,
      };

      // 创建后端会话
      const session = await authApi.createSession(response.credential);
      
      // 存储 token
      localStorage.setItem('auth_token', session.token);

      // 触发登录成功事件
      window.dispatchEvent(new CustomEvent('googleSignIn', { 
        detail: { user: googleUser, session } 
      }));
    } catch (error) {
      window.dispatchEvent(new CustomEvent('googleSignInError', { 
        detail: { error } 
      }));
    }
  }

  private parseJwt(token: string): any {
    try {
      const base64Url = token.split('.')[1];
      const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/');
      const jsonPayload = decodeURIComponent(
        atob(base64)
          .split('')
          .map(c => '%' + ('00' + c.charCodeAt(0).toString(16)).slice(-2))
          .join('')
      );
      return JSON.parse(jsonPayload);
    } catch (error) {
      return {};
    }
  }

  async signIn(): Promise<{ user: GoogleUser; session: any }> {
    if (!this.isInitialized) {
      await this.initialize();
    }

    return new Promise((resolve, reject) => {
      // 监听登录成功事件
      const handleSuccess = (event: any) => {
        window.removeEventListener('googleSignIn', handleSuccess);
        window.removeEventListener('googleSignInError', handleError);
        resolve(event.detail);
      };

      // 监听登录失败事件
      const handleError = (event: any) => {
        window.removeEventListener('googleSignIn', handleSuccess);
        window.removeEventListener('googleSignInError', handleError);
        reject(event.detail.error);
      };

      // 检查是否已经有pending的登录事件
      const checkForPendingEvent = () => {
        const token = localStorage.getItem('auth_token');
        if (token) {
          // 如果有token，尝试获取用户信息来验证
          authApi.getCurrentUser()
            .then(user => {
              // 从localStorage获取token，构造session对象
              const session = {
                token: token,
                user: user
              };
              // 构造GoogleUser对象
              const googleUser = {
                id: user.handle,
                email: user.handle,
                name: user.handle,
                picture: ''
              };
              resolve({ user: googleUser, session });
            })
            .catch(() => {
              // Token无效，继续正常登录流程
              window.addEventListener('googleSignIn', handleSuccess);
              window.addEventListener('googleSignInError', handleError);
              window.google.accounts.id.prompt();
            });
        } else {
          window.addEventListener('googleSignIn', handleSuccess);
          window.addEventListener('googleSignInError', handleError);
          window.google.accounts.id.prompt();
        }
      };

      // 延迟检查，给Google登录流程一些时间
      setTimeout(checkForPendingEvent, 100);
    });
  }

  async signOut(): Promise<void> {
    if (window.google && this.isInitialized) {
      window.google.accounts.id.disableAutoSelect();
    }
    localStorage.removeItem('auth_token');
  }

  async getCurrentUser(): Promise<GoogleUser | null> {
    const token = localStorage.getItem('auth_token');
    if (!token) return null;

    try {
      // 从后端获取当前用户信息
      const user = await authApi.getCurrentUser();
      return {
        id: user.handle, // 使用 handle 作为 id
        email: user.handle,
        name: user.handle,
        picture: '', // 后端没有存储头像信息
      };
    } catch (error) {
      return null;
    }
  }

  // 渲染 Google 登录按钮
  renderButton(elementId: string): void {
    if (!this.isInitialized) {
      return;
    }

    window.google.accounts.id.renderButton(
      document.getElementById(elementId),
      {
        theme: 'outline',
        size: 'large',
        text: 'signin_with',
        shape: 'rectangular',
        logo_alignment: 'left',
        width: '100%',
      }
    );
  }
}

export const googleAuth = new GoogleAuthService();
