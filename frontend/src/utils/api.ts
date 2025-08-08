import httpClient, { ApiError } from '../services/httpClient';
import type { 
  LoginResponse, 
  User, 
  Application, 
  CreateApplicationRequest, 
  UpdateApplicationRequest,
  Analytics,
  Screening,
  Interview,
  ActivityData
} from '../types';

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8000';

// Enhanced error handling wrapper
const withErrorHandling = async <T>(operation: () => Promise<T>): Promise<T> => {
  try {
    return await operation();
  } catch (error) {
    const apiError = error as ApiError;
    console.error('API Error:', {
      message: apiError.message,
      status: apiError.status,
      code: apiError.code,
    });
    throw new Error(apiError.message || 'An unexpected error occurred');
  }
};

export const authApi = {
  register: async (data: { email: string; password: string; first_name: string; last_name: string; adminCode?: string }): Promise<User> => {
    return withErrorHandling(() => httpClient.post<User>('/auth/register', { 
      email: data.email, 
      password: data.password,
      first_name: data.first_name,
      last_name: data.last_name,
      admin_code: data.adminCode 
    }));
  },

  registerAdmin: async (data: { email: string; password: string; role?: 'student' | 'admin' }): Promise<User> => {
    return withErrorHandling(() => httpClient.post<User>('/admin/register', data));
  },

  login: async (data: { email: string; password: string }): Promise<LoginResponse> => {
    return withErrorHandling(() => httpClient.post<LoginResponse>('/auth/login', data));
  },
};

export const applicationsApi = {
  getApplications: async (): Promise<Application[]> => {
    return withErrorHandling(() => httpClient.get<Application[]>('/applications'));
  },

  getApplication: async (id: number): Promise<Application> => {
    return withErrorHandling(() => httpClient.get<Application>(`/applications/${id}`));
  },

  createApplication: async (data: CreateApplicationRequest): Promise<Application> => {
    return withErrorHandling(() => httpClient.post<Application>('/applications', data));
  },

  updateApplication: async (id: number, data: UpdateApplicationRequest): Promise<Application> => {
    return withErrorHandling(() => httpClient.put<Application>(`/applications/${id}`, data));
  },

  deleteApplication: async (id: number): Promise<void> => {
    return withErrorHandling(() => httpClient.delete<void>(`/applications/${id}`));
  },

  uploadScreening: async (id: number, formData: FormData): Promise<Screening> => {
    return withErrorHandling(() => httpClient.upload<Screening>(`/applications/${id}/screening`, formData));
  },

  uploadInterview: async (id: number, formData: FormData): Promise<Interview> => {
    return withErrorHandling(() => httpClient.upload<Interview>(`/applications/${id}/interview`, formData));
  },

  getActivity: async (): Promise<ActivityData[]> => {
    return withErrorHandling(() => httpClient.get<ActivityData[]>('/applications/activity'));
  },
};

export const adminApi = {
  getAnalytics: async (params?: { company?: string; status?: string; days_stale?: number }): Promise<Analytics> => {
    return withErrorHandling(() => httpClient.get<Analytics>('/admin/analytics', params));
  },

  getAllStudents: async (): Promise<User[]> => {
    return withErrorHandling(() => httpClient.get<User[]>('/admin/students'));
  },

  getAllApplications: async (params?: { company?: string; status?: string }): Promise<Application[]> => {
    return withErrorHandling(() => httpClient.get<Application[]>('/admin/applications', params));
  },

  getActivity: async (): Promise<ActivityData[]> => {
    return withErrorHandling(() => httpClient.get<ActivityData[]>('/admin/activity'));
  },

  getUserActivity: async (userId: number): Promise<ActivityData[]> => {
    return withErrorHandling(() => httpClient.get<ActivityData[]>(`/admin/users/${userId}/activity`));
  },
};

export const filesApi = {
  getFileUrl: (filename: string): string => {
    const token = localStorage.getItem('token');
    if (token) {
      // Use the token-based download endpoint for both students and admins
      return `${API_BASE_URL}/download/${filename}?token=${encodeURIComponent(token)}`;
    }
    return `${API_BASE_URL}/files/${filename}`;
  },

  downloadFile: async (filename: string): Promise<Blob> => {
    return withErrorHandling(async () => {
      const token = localStorage.getItem('token');
      // Using axios directly for blob response type as it's not in our HTTP client interface
      const response = await fetch(`${API_BASE_URL}/download/${filename}?token=${encodeURIComponent(token || '')}`, {
        headers: {
          'Authorization': `Bearer ${token}`
        }
      });
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }
      return response.blob();
    });
  },
};