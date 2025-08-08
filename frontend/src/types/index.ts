export interface User {
  id: number;
  email: string;
  first_name: string;
  last_name: string;
  role: 'student' | 'admin';
  created_at: string;
}

export interface LoginResponse {
  token: string;
  user: User;
}

export interface Application {
  id: number;
  user_id: number;
  company_name: string;
  job_url?: string;
  application_date: string;
  status: 'waiting' | 'rejected' | 'next_stage' | 'ignored';
  created_at: string;
  updated_at: string;
  screening?: Screening;
  interview?: Interview;
}

export interface Screening {
  id: number;
  application_id: number;
  file_path?: string;
  screening_date?: string;
  result?: 'passed' | 'failed';
  created_at: string;
  updated_at: string;
}

export interface Interview {
  id: number;
  application_id: number;
  file_path?: string;
  interview_date?: string;
  result?: 'passed' | 'failed';
  created_at: string;
  updated_at: string;
}

export interface CreateApplicationRequest {
  company_name: string;
  job_url?: string;
  application_date: string;
}

export interface UpdateApplicationRequest {
  company_name?: string;
  job_url?: string;
  application_date?: string;
  status?: 'waiting' | 'rejected' | 'next_stage' | 'ignored';
}

export interface Analytics {
  total_students: number;
  total_applications: number;
  status_breakdown: Record<string, number>;
  company_stats: CompanyStats[];
  popular_job_urls: JobUrlStats[];
  stale_applications: Application[];
  screening_stats: ScreeningStats;
  interview_stats: InterviewStats;
  daily_stats: DailyStat[];
  success_rate: SuccessRateStats;
  response_times: ResponseTimeStats;
  top_performing_students: StudentPerformance[];
}

export interface ScreeningStats {
  total_screenings: number;
  passed: number;
  failed: number;
  pending: number;
}

export interface InterviewStats {
  total_interviews: number;
  passed: number;
  failed: number;
  pending: number;
}

export interface CompanyStats {
  company: string;
  application_count: number;
  unique_students: number;
}

export interface JobUrlStats {
  job_url: string;
  application_count: number;
  unique_students: number;
}

export interface DailyStat {
  date: string;
  applications_count: number;
  screenings_count: number;
  interviews_count: number;
}

export interface SuccessRateStats {
  overall_success_rate: number;
  screening_to_interview_rate: number;
  interview_success_rate: number;
  applications_with_urls: number;
  applications_without_urls: number;
}

export interface ResponseTimeStats {
  avg_days_to_screening: number;
  avg_days_to_interview: number;
  fastest_screening_days: number;
  slowest_screening_days: number;
}

export interface StudentPerformance {
  student_email: string;
  student_name: string;
  total_applications: number;
  screenings_passed: number;
  interviews_passed: number;
  success_rate: number;
}

export interface ActivityData {
  date: string;
  applications_count: number;
  screenings_count: number;
  interviews_count: number;
  total_activity: number;
}

