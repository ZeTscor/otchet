import axios, { AxiosInstance, AxiosResponse, AxiosError, InternalAxiosRequestConfig } from 'axios';

export interface HttpClientConfig {
  baseURL: string;
  timeout?: number;
  retryAttempts?: number;
  retryDelay?: number;
}

export interface ApiError {
  message: string;
  status: number;
  code?: string;
  details?: any;
}

export class HttpClient {
  private client: AxiosInstance;
  private retryAttempts: number;
  private retryDelay: number;

  constructor(config: HttpClientConfig) {
    this.retryAttempts = config.retryAttempts ?? 3;
    this.retryDelay = config.retryDelay ?? 1000;

    this.client = axios.create({
      baseURL: config.baseURL,
      timeout: config.timeout ?? 30000,
      headers: {
        'Content-Type': 'application/json',
      },
    });

    this.setupInterceptors();
  }

  private setupInterceptors(): void {
    // Request interceptor
    this.client.interceptors.request.use(
      (config: InternalAxiosRequestConfig) => {
        const token = localStorage.getItem('token');
        if (token && config.headers) {
          config.headers.Authorization = `Bearer ${token}`;
        }

        // Log request for debugging
        this.logRequest(config);
        return config;
      },
      (error) => {
        this.logError('Request Error', error);
        return Promise.reject(error);
      }
    );

    // Response interceptor
    this.client.interceptors.response.use(
      (response: AxiosResponse) => {
        this.logResponse(response);
        return response;
      },
      async (error: AxiosError) => {
        if (error.response?.status === 401) {
          localStorage.removeItem('token');
          localStorage.removeItem('user');
          window.location.href = '/login';
          return Promise.reject(new Error('Authentication failed'));
        }

        // Retry logic for network errors or 5xx server errors
        if (this.shouldRetry(error)) {
          return this.retryRequest(error);
        }

        this.logError('Response Error', error);
        return Promise.reject(this.transformError(error));
      }
    );
  }

  private shouldRetry(error: AxiosError): boolean {
    const status = error.response?.status;
    return (
      !status || // Network error
      status >= 500 || // Server error
      status === 408 || // Request timeout
      status === 429 // Too many requests
    );
  }

  private async retryRequest(error: AxiosError, attempt: number = 1): Promise<AxiosResponse> {
    if (attempt > this.retryAttempts) {
      throw this.transformError(error);
    }

    console.log(`Retrying request (attempt ${attempt}/${this.retryAttempts})`);
    
    await this.delay(this.retryDelay * Math.pow(2, attempt - 1)); // Exponential backoff
    
    try {
      return await this.client.request(error.config!);
    } catch (retryError) {
      if (this.shouldRetry(retryError as AxiosError)) {
        return this.retryRequest(retryError as AxiosError, attempt + 1);
      }
      throw this.transformError(retryError as AxiosError);
    }
  }

  private delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  private transformError(error: AxiosError): ApiError {
    const status = error.response?.status ?? 0;
    const message = (error.response?.data as any)?.message ?? error.message ?? 'Unknown error occurred';
    
    return {
      message,
      status,
      code: error.code,
      details: error.response?.data,
    };
  }

  private logRequest(config: InternalAxiosRequestConfig): void {
    if (import.meta.env.DEV) {
      console.log(`🚀 ${config.method?.toUpperCase()} ${config.url}`, {
        params: config.params,
        data: config.data,
      });
    }
  }

  private logResponse(response: AxiosResponse): void {
    if (import.meta.env.DEV) {
      console.log(`✅ ${response.status} ${response.config.method?.toUpperCase()} ${response.config.url}`, {
        duration: response.headers['x-response-time'],
        data: response.data,
      });
    }
  }

  private logError(type: string, error: any): void {
    if (import.meta.env.DEV) {
      console.error(`❌ ${type}:`, {
        message: error.message,
        status: error.response?.status,
        url: error.config?.url,
        method: error.config?.method,
      });
    }
  }

  // HTTP methods
  async get<T = any>(url: string, params?: any): Promise<T> {
    const response = await this.client.get<T>(url, { params });
    return response.data;
  }

  async post<T = any>(url: string, data?: any): Promise<T> {
    const response = await this.client.post<T>(url, data);
    return response.data;
  }

  async put<T = any>(url: string, data?: any): Promise<T> {
    const response = await this.client.put<T>(url, data);
    return response.data;
  }

  async delete<T = any>(url: string): Promise<T> {
    const response = await this.client.delete<T>(url);
    return response.data;
  }

  async upload<T = any>(url: string, formData: FormData): Promise<T> {
    const response = await this.client.post<T>(url, formData, {
      headers: {
        'Content-Type': 'multipart/form-data',
      },
    });
    return response.data;
  }
}

// Create singleton instance
const httpClient = new HttpClient({
  baseURL: import.meta.env.VITE_API_BASE_URL || 'http://localhost:8000',
  timeout: 30000,
  retryAttempts: 3,
  retryDelay: 1000,
});

export default httpClient;