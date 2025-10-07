import React, { useState } from 'react';
import { adminApi } from '../lib/api';
import { RoleUpdateRequest } from '../types';
import { 
  Shield, 
  Users, 
  Crown, 
  AlertCircle,
  CheckCircle,
  Save,
  RefreshCw
} from 'lucide-react';

const AdminPage: React.FC = () => {
  const [formData, setFormData] = useState<RoleUpdateRequest>({
    handle: '',
    channel: 'web',
    role: 'user'
  });
  const [isLoading, setIsLoading] = useState(false);
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!formData.handle.trim()) {
      setMessage({ type: 'error', text: '请输入用户邮箱' });
      return;
    }

    try {
      setIsLoading(true);
      setMessage(null);
      
      await adminApi.updateRole(formData);
      setMessage({ type: 'success', text: '用户角色更新成功' });
      
      // Reset form
      setFormData({
        handle: '',
        channel: 'web',
        role: 'user'
      });
    } catch (error: any) {
      setMessage({ 
        type: 'error', 
        text: error.response?.data?.message || '更新失败，请重试' 
      });
    } finally {
      setIsLoading(false);
    }
  };

  const handleInputChange = (field: keyof RoleUpdateRequest, value: string) => {
    setFormData(prev => ({
      ...prev,
      [field]: value
    }));
  };

  const roleOptions = [
    { value: 'user', label: '普通用户', description: '基础权限，可领取默认数量代币' },
    { value: 'privileged', label: '特权用户', description: '更高权限，可领取更多代币' },
    { value: 'admin', label: '管理员', description: '最高权限，可管理其他用户' }
  ];

  const channelOptions = [
    { value: 'web', label: 'Web' },
    { value: 'telegram', label: 'Telegram' },
    { value: 'discord', label: 'Discord' }
  ];

  return (
    <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      {/* Header */}
      <div className="text-center mb-8">
        <div className="mx-auto h-16 w-16 bg-yellow-100 rounded-full flex items-center justify-center mb-4">
          <Shield className="h-8 w-8 text-yellow-600" />
        </div>
        <h1 className="text-3xl font-bold text-gray-900 mb-2">
          管理后台
        </h1>
        <p className="text-gray-600">
          管理用户角色和权限设置
        </p>
      </div>

      {/* Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
        <div className="card p-6 text-center">
          <Users className="h-8 w-8 text-blue-500 mx-auto mb-2" />
          <div className="text-2xl font-bold text-gray-900">用户管理</div>
          <div className="text-sm text-gray-600">管理用户角色和权限</div>
        </div>
        <div className="card p-6 text-center">
          <Crown className="h-8 w-8 text-purple-500 mx-auto mb-2" />
          <div className="text-2xl font-bold text-gray-900">权限控制</div>
          <div className="text-sm text-gray-600">设置用户权限级别</div>
        </div>
        <div className="card p-6 text-center">
          <Shield className="h-8 w-8 text-green-500 mx-auto mb-2" />
          <div className="text-2xl font-bold text-gray-900">安全设置</div>
          <div className="text-sm text-gray-600">确保系统安全</div>
        </div>
      </div>

      {/* Role Management Form */}
      <div className="card p-6">
        <h2 className="text-xl font-semibold text-gray-900 mb-6">更新用户角色</h2>
        
        {message && (
          <div className={`mb-6 p-4 rounded-md ${
            message.type === 'success' 
              ? 'bg-green-50 border border-green-200' 
              : 'bg-red-50 border border-red-200'
          }`}>
            <div className="flex">
              {message.type === 'success' ? (
                <CheckCircle className="h-5 w-5 text-green-400" />
              ) : (
                <AlertCircle className="h-5 w-5 text-red-400" />
              )}
              <div className="ml-3">
                <p className={`text-sm font-medium ${
                  message.type === 'success' ? 'text-green-800' : 'text-red-800'
                }`}>
                  {message.text}
                </p>
              </div>
            </div>
          </div>
        )}

        <form onSubmit={handleSubmit} className="space-y-6">
          {/* User Handle */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              用户邮箱 <span className="text-red-500">*</span>
            </label>
            <input
              type="email"
              value={formData.handle}
              onChange={(e) => handleInputChange('handle', e.target.value)}
              placeholder="user@example.com"
              className="input w-full"
              required
            />
            <p className="text-xs text-gray-500 mt-1">
              请输入用户的完整邮箱地址
            </p>
          </div>

          {/* Channel */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              渠道
            </label>
            <select
              value={formData.channel}
              onChange={(e) => handleInputChange('channel', e.target.value)}
              className="input w-full"
            >
              {channelOptions.map(option => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </div>

          {/* Role */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              角色
            </label>
            <div className="space-y-3">
              {roleOptions.map(option => (
                <div key={option.value} className="flex items-start">
                  <input
                    type="radio"
                    id={option.value}
                    name="role"
                    value={option.value}
                    checked={formData.role === option.value}
                    onChange={(e) => handleInputChange('role', e.target.value)}
                    className="mt-1 h-4 w-4 text-primary-600 focus:ring-primary-500 border-gray-300"
                  />
                  <label htmlFor={option.value} className="ml-3">
                    <div className="text-sm font-medium text-gray-900">
                      {option.label}
                    </div>
                    <div className="text-sm text-gray-500">
                      {option.description}
                    </div>
                  </label>
                </div>
              ))}
            </div>
          </div>

          {/* Submit Button */}
          <div className="flex justify-end">
            <button
              type="submit"
              disabled={isLoading}
              className="btn btn-primary btn-lg disabled:opacity-50"
            >
              {isLoading ? (
                <>
                  <RefreshCw className="h-4 w-4 mr-2 animate-spin" />
                  更新中...
                </>
              ) : (
                <>
                  <Save className="h-4 w-4 mr-2" />
                  更新角色
                </>
              )}
            </button>
          </div>
        </form>
      </div>

      {/* Admin Info */}
      <div className="mt-8 card p-6 bg-yellow-50 border-yellow-200">
        <div className="flex">
          <Shield className="h-5 w-5 text-yellow-600 mt-0.5" />
          <div className="ml-3">
            <h3 className="text-sm font-medium text-yellow-800">
              管理员权限说明
            </h3>
            <div className="mt-2 text-sm text-yellow-700">
              <ul className="list-disc list-inside space-y-1">
                <li>只有管理员可以访问此页面</li>
                <li>可以修改任何用户的角色和权限</li>
                <li>请谨慎操作，确保用户身份正确</li>
                <li>角色更改将立即生效</li>
              </ul>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default AdminPage;
