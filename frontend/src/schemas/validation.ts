import { z } from 'zod';

// Date validation schema for YYYY-MM-DD format
const DateSchema = z.string().regex(/^\d{4}-\d{2}-\d{2}$/, {
  message: "Дата должна быть в формате ГГГГ-ММ-ДД"
});

// URL validation schema
const UrlSchema = z.string().url({ message: "Пожалуйста, введите корректный URL" }).optional();

// Application schemas
export const CreateApplicationSchema = z.object({
  company_name: z.string().min(1, { message: "Название компании обязательно" }),
  job_url: UrlSchema,
  application_date: DateSchema,
});

export const UpdateApplicationSchema = z.object({
  company_name: z.string().min(1, { message: "Название компании обязательно" }).optional(),
  job_url: UrlSchema,
  application_date: DateSchema.optional(),
  status: z.enum(['waiting', 'rejected', 'next_stage', 'ignored']).optional(),
});

// User schemas
export const RegisterSchema = z.object({
  email: z.string().email({ message: "Пожалуйста, введите корректный адрес электронной почты" }),
  password: z.string().min(6, { message: "Пароль должен содержать не менее 6 символов" }),
  confirmPassword: z.string(),
}).refine((data) => data.password === data.confirmPassword, {
  message: "Пароли не совпадают",
  path: ["confirmPassword"],
});

export const LoginSchema = z.object({
  email: z.string().email({ message: "Пожалуйста, введите корректный адрес электронной почты" }),
  password: z.string().min(1, { message: "Пароль обязателен" }),
});

// File upload schemas
const MAX_FILE_SIZE = 500 * 1024 * 1024; // 500MB
const ACCEPTED_AUDIO_TYPES = ['audio/mpeg', 'audio/wav', 'audio/ogg', 'audio/mp4', 'audio/m4a'];
const ACCEPTED_VIDEO_TYPES = ['video/mp4', 'video/webm', 'video/quicktime', 'video/x-msvideo'];
const ACCEPTED_FILE_TYPES = [...ACCEPTED_AUDIO_TYPES, ...ACCEPTED_VIDEO_TYPES];

export const FileSchema = z
  .instanceof(File)
  .refine((file) => file.size <= MAX_FILE_SIZE, {
    message: "Размер файла должен быть меньше 500МБ",
  })
  .refine((file) => ACCEPTED_FILE_TYPES.includes(file.type), {
    message: "Разрешены только аудио и видео файлы",
  });

export const ScreeningUploadSchema = z.object({
  screening_date: DateSchema.optional(),
  screening_status: z.enum(['passed', 'failed']).optional(),
  file: FileSchema,
});

export const InterviewUploadSchema = z.object({
  interview_date: DateSchema.optional(),
  interview_status: z.enum(['passed', 'failed']).optional(),
  file: FileSchema,
});

// Type inference for TypeScript
export type CreateApplicationForm = z.infer<typeof CreateApplicationSchema>;
export type UpdateApplicationForm = z.infer<typeof UpdateApplicationSchema>;
export type RegisterForm = z.infer<typeof RegisterSchema>;
export type LoginForm = z.infer<typeof LoginSchema>;
export type ScreeningUploadForm = z.infer<typeof ScreeningUploadSchema>;
export type InterviewUploadForm = z.infer<typeof InterviewUploadSchema>;

// Helper function to format validation errors
export function formatValidationErrors(error: z.ZodError): Record<string, string> {
  const errors: Record<string, string> = {};
  error.issues.forEach((err) => {
    const path = err.path.join('.');
    errors[path] = err.message;
  });
  return errors;
}