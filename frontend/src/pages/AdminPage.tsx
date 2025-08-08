import { useState, useEffect } from 'react';
import { adminApi } from '../utils/api';
import type { Analytics } from '../types';
import { 
  UserGroupIcon, 
  BuildingOfficeIcon,
  DocumentCheckIcon,
  ChatBubbleLeftRightIcon,
  LinkIcon,
  ExclamationTriangleIcon
} from '@heroicons/react/24/outline';

export default function AdminPage() {
  const [analytics, setAnalytics] = useState<Analytics | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string>('');

  useEffect(() => {
    fetchAnalytics();
  }, []);

  const fetchAnalytics = async () => {
    try {
      const data = await adminApi.getAnalytics();
      setAnalytics(data);
    } catch (err: any) {
      setError('Ошибка загрузки аналитики');
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="flex justify-center items-center h-64">
        <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-indigo-500"></div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-50 dark:bg-red-900/20 border border-red-300 dark:border-red-800 text-red-700 dark:text-red-400 px-4 py-3 rounded">
        {error}
      </div>
    );
  }

  if (!analytics) return null;

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-gray-900 dark:text-gray-100">Панель администратора</h1>
        <p className="mt-2 text-sm text-gray-700 dark:text-gray-300">
          Полная аналитика по заявкам студентов, скринингам и интервью
        </p>
      </div>

      {/* Overview Stats */}
      <div className="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-4">
        <div className="bg-white dark:bg-gray-800 overflow-hidden shadow rounded-lg">
          <div className="p-5">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <UserGroupIcon className="h-6 w-6 text-gray-400" />
              </div>
              <div className="ml-5 w-0 flex-1">
                <dl>
                  <dt className="text-sm font-medium text-gray-500 dark:text-gray-400 truncate">Студенты</dt>
                  <dd className="text-lg font-medium text-gray-900 dark:text-gray-100">{analytics.total_students}</dd>
                </dl>
              </div>
            </div>
          </div>
        </div>

        <div className="bg-white dark:bg-gray-800 overflow-hidden shadow rounded-lg">
          <div className="p-5">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <BuildingOfficeIcon className="h-6 w-6 text-gray-400" />
              </div>
              <div className="ml-5 w-0 flex-1">
                <dl>
                  <dt className="text-sm font-medium text-gray-500 dark:text-gray-400 truncate">Заявки</dt>
                  <dd className="text-lg font-medium text-gray-900 dark:text-gray-100">{analytics.total_applications}</dd>
                </dl>
              </div>
            </div>
          </div>
        </div>

        <div className="bg-white dark:bg-gray-800 overflow-hidden shadow rounded-lg">
          <div className="p-5">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <DocumentCheckIcon className="h-6 w-6 text-blue-400" />
              </div>
              <div className="ml-5 w-0 flex-1">
                <dl>
                  <dt className="text-sm font-medium text-gray-500 dark:text-gray-400 truncate">Скрининги</dt>
                  <dd className="text-lg font-medium text-gray-900 dark:text-gray-100">{analytics.screening_stats.total_screenings}</dd>
                  <dd className="text-xs text-green-600 dark:text-green-400">{analytics.screening_stats.passed} пройдены</dd>
                </dl>
              </div>
            </div>
          </div>
        </div>

        <div className="bg-white dark:bg-gray-800 overflow-hidden shadow rounded-lg">
          <div className="p-5">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <ChatBubbleLeftRightIcon className="h-6 w-6 text-green-400" />
              </div>
              <div className="ml-5 w-0 flex-1">
                <dl>
                  <dt className="text-sm font-medium text-gray-500 dark:text-gray-400 truncate">Интервью</dt>
                  <dd className="text-lg font-medium text-gray-900 dark:text-gray-100">{analytics.interview_stats.total_interviews}</dd>
                  <dd className="text-xs text-green-600 dark:text-green-400">{analytics.interview_stats.passed} пройдены</dd>
                </dl>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Screening & Interview Stats */}
      <div className="grid grid-cols-1 gap-5 lg:grid-cols-2">
        <div className="bg-white shadow overflow-hidden sm:rounded-lg">
          <div className="px-4 py-5 sm:px-6">
            <h3 className="text-lg leading-6 font-medium text-gray-900 dark:text-gray-100">Статистика скрининга</h3>
            <p className="mt-1 max-w-2xl text-sm text-gray-500 dark:text-gray-400">Результаты прохождения скрининга</p>
          </div>
          <div className="border-t border-gray-200 dark:border-gray-700">
            <dl>
              <div className="bg-gray-50 dark:bg-gray-800 px-4 py-5 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-6">
                <dt className="text-sm font-medium text-gray-500">Всего скринингов</dt>
                <dd className="mt-1 text-sm text-gray-900 dark:text-gray-100 sm:mt-0 sm:col-span-2">{analytics.screening_stats.total_screenings}</dd>
              </div>
              <div className="bg-white dark:bg-gray-700 px-4 py-5 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-6">
                <dt className="text-sm font-medium text-gray-500">Пройдено</dt>
                <dd className="mt-1 text-sm text-green-600 dark:text-green-400 font-medium sm:mt-0 sm:col-span-2">
                  {analytics.screening_stats.passed} ({Math.round((analytics.screening_stats.passed / analytics.screening_stats.total_screenings) * 100)}%)
                </dd>
              </div>
              <div className="bg-gray-50 dark:bg-gray-800 px-4 py-5 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-6">
                <dt className="text-sm font-medium text-gray-500">Провалено</dt>
                <dd className="mt-1 text-sm text-red-600 dark:text-red-400 font-medium sm:mt-0 sm:col-span-2">
                  {analytics.screening_stats.failed} ({Math.round((analytics.screening_stats.failed / analytics.screening_stats.total_screenings) * 100)}%)
                </dd>
              </div>
              <div className="bg-white dark:bg-gray-700 px-4 py-5 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-6">
                <dt className="text-sm font-medium text-gray-500">В ожидании</dt>
                <dd className="mt-1 text-sm text-yellow-600 dark:text-yellow-400 font-medium sm:mt-0 sm:col-span-2">
                  {analytics.screening_stats.pending} ({Math.round((analytics.screening_stats.pending / analytics.screening_stats.total_screenings) * 100)}%)
                </dd>
              </div>
            </dl>
          </div>
        </div>

        <div className="bg-white shadow overflow-hidden sm:rounded-lg">
          <div className="px-4 py-5 sm:px-6">
            <h3 className="text-lg leading-6 font-medium text-gray-900 dark:text-gray-100">Статистика интервью</h3>
            <p className="mt-1 max-w-2xl text-sm text-gray-500 dark:text-gray-400">Результаты прохождения интервью</p>
          </div>
          <div className="border-t border-gray-200 dark:border-gray-700">
            <dl>
              <div className="bg-gray-50 dark:bg-gray-800 px-4 py-5 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-6">
                <dt className="text-sm font-medium text-gray-500">Всего интервью</dt>
                <dd className="mt-1 text-sm text-gray-900 dark:text-gray-100 sm:mt-0 sm:col-span-2">{analytics.interview_stats.total_interviews}</dd>
              </div>
              <div className="bg-white dark:bg-gray-700 px-4 py-5 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-6">
                <dt className="text-sm font-medium text-gray-500">Пройдено</dt>
                <dd className="mt-1 text-sm text-green-600 dark:text-green-400 font-medium sm:mt-0 sm:col-span-2">
                  {analytics.interview_stats.passed} ({analytics.interview_stats.total_interviews > 0 ? Math.round((analytics.interview_stats.passed / analytics.interview_stats.total_interviews) * 100) : 0}%)
                </dd>
              </div>
              <div className="bg-gray-50 dark:bg-gray-800 px-4 py-5 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-6">
                <dt className="text-sm font-medium text-gray-500">Провалено</dt>
                <dd className="mt-1 text-sm text-red-600 dark:text-red-400 font-medium sm:mt-0 sm:col-span-2">
                  {analytics.interview_stats.failed} ({analytics.interview_stats.total_interviews > 0 ? Math.round((analytics.interview_stats.failed / analytics.interview_stats.total_interviews) * 100) : 0}%)
                </dd>
              </div>
              <div className="bg-white dark:bg-gray-700 px-4 py-5 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-6">
                <dt className="text-sm font-medium text-gray-500">В ожидании</dt>
                <dd className="mt-1 text-sm text-yellow-600 dark:text-yellow-400 font-medium sm:mt-0 sm:col-span-2">
                  {analytics.interview_stats.pending} ({analytics.interview_stats.total_interviews > 0 ? Math.round((analytics.interview_stats.pending / analytics.interview_stats.total_interviews) * 100) : 0}%)
                </dd>
              </div>
            </dl>
          </div>
        </div>
      </div>

      {/* Status Breakdown */}
      <div className="bg-white shadow overflow-hidden sm:rounded-lg">
        <div className="px-4 py-5 sm:px-6">
          <h3 className="text-lg leading-6 font-medium text-gray-900">Распределение по статусам</h3>
          <p className="mt-1 max-w-2xl text-sm text-gray-500">Текущее состояние всех заявок</p>
        </div>
        <div className="border-t border-gray-200">
          <dl>
            {Object.entries(analytics.status_breakdown).map(([status, count], index) => {
              const statusNames: Record<string, string> = {
                'waiting': 'Ожидание',
                'next_stage': 'Следующий этап', 
                'rejected': 'Отклонена',
                'ignored': 'Игнорируется'
              };
              const statusColors: Record<string, string> = {
                'waiting': 'text-yellow-600 dark:text-yellow-400',
                'next_stage': 'text-blue-600 dark:text-blue-400',
                'rejected': 'text-red-600 dark:text-red-400', 
                'ignored': 'text-gray-600 dark:text-gray-400'
              };
              return (
                <div key={status} className={`${index % 2 === 0 ? 'bg-gray-50 dark:bg-gray-800' : 'bg-white dark:bg-gray-700'} px-4 py-5 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-6`}>
                  <dt className="text-sm font-medium text-gray-500 dark:text-gray-400">
                    {statusNames[status] || status}
                  </dt>
                  <dd className={`mt-1 text-sm font-medium sm:mt-0 sm:col-span-2 ${statusColors[status] || 'text-gray-900 dark:text-gray-100'}`}>
                    {count} заявок ({Math.round((count / analytics.total_applications) * 100)}%)
                  </dd>
                </div>
              );
            })}
          </dl>
        </div>
      </div>

      {/* Companies and Job URLs Grid */}
      <div className="grid grid-cols-1 gap-5 lg:grid-cols-2">
        {/* Top Companies */}
        <div className="bg-white shadow overflow-hidden sm:rounded-lg">
          <div className="px-4 py-5 sm:px-6">
            <h3 className="text-lg leading-6 font-medium text-gray-900 dark:text-gray-100">Популярные компании</h3>
            <p className="mt-1 max-w-2xl text-sm text-gray-500 dark:text-gray-400">
              Компании с наибольшим количеством заявок
            </p>
          </div>
          <div className="border-t border-gray-200 dark:border-gray-700">
            <ul className="divide-y divide-gray-200 dark:divide-gray-700">
              {analytics.company_stats.slice(0, 8).map((company) => (
                <li key={company.company} className="px-4 py-4 flex items-center justify-between">
                  <div>
                    <p className="text-sm font-medium text-gray-900 dark:text-gray-100">{company.company}</p>
                    <p className="text-sm text-gray-500 dark:text-gray-400">{company.unique_students} студентов</p>
                  </div>
                  <div className="text-sm text-gray-900 dark:text-gray-100 font-medium">
                    {company.application_count} заявок
                  </div>
                </li>
              ))}
            </ul>
          </div>
        </div>

        {/* Popular Job URLs */}
        <div className="bg-white shadow overflow-hidden sm:rounded-lg">
          <div className="px-4 py-5 sm:px-6">
            <h3 className="text-lg leading-6 font-medium text-gray-900 dark:text-gray-100">Популярные вакансии</h3>
            <p className="mt-1 max-w-2xl text-sm text-gray-500 dark:text-gray-400">
              Наиболее часто встречающиеся URL вакансий
            </p>
          </div>
          <div className="border-t border-gray-200 dark:border-gray-700">
            <ul className="divide-y divide-gray-200 dark:divide-gray-700">
              {analytics.popular_job_urls.slice(0, 8).map((jobUrl, index) => (
                <li key={index} className="px-4 py-4 flex items-center justify-between">
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center">
                      <LinkIcon className="h-4 w-4 text-gray-400 mr-2 flex-shrink-0" />
                      <a 
                        href={jobUrl.job_url} 
                        target="_blank" 
                        rel="noopener noreferrer"
                        className="text-sm text-indigo-600 dark:text-indigo-400 hover:text-indigo-500 dark:hover:text-indigo-300 truncate"
                        title={jobUrl.job_url}
                      >
                        {jobUrl.job_url.length > 40 ? `${jobUrl.job_url.substring(0, 40)}...` : jobUrl.job_url}
                      </a>
                    </div>
                    <p className="text-sm text-gray-500 dark:text-gray-400">{jobUrl.unique_students} студентов</p>
                  </div>
                  <div className="text-sm text-gray-900 dark:text-gray-100 font-medium ml-4">
                    {jobUrl.application_count} заявок
                  </div>
                </li>
              ))}
            </ul>
          </div>
        </div>
      </div>

      {/* Stale Applications Alert */}
      {analytics.stale_applications.length > 0 && (
        <div className="bg-yellow-50 dark:bg-yellow-900/20 border-l-4 border-yellow-400 dark:border-yellow-600 p-4">
          <div className="flex">
            <div className="flex-shrink-0">
              <ExclamationTriangleIcon className="h-5 w-5 text-yellow-400" />
            </div>
            <div className="ml-3">
              <h3 className="text-sm font-medium text-yellow-800 dark:text-yellow-400">Устаревшие заявки</h3>
              <div className="mt-2 text-sm text-yellow-700 dark:text-yellow-400">
                <p>
                  Найдено <strong>{analytics.stale_applications.length}</strong> заявок без обновлений более 7 дней.
                </p>
                <div className="mt-2">
                  <div className="max-h-32 overflow-y-auto">
                    {analytics.stale_applications.slice(0, 5).map((app) => (
                      <div key={app.id} className="text-xs text-yellow-600 dark:text-yellow-400 mt-1">
                        • {app.company_name} (студент ID: {app.user_id}) - {new Date(app.updated_at).toLocaleDateString('ru-RU')}
                      </div>
                    ))}
                    {analytics.stale_applications.length > 5 && (
                      <div className="text-xs text-yellow-600 dark:text-yellow-400 mt-1">
                        ... и еще {analytics.stale_applications.length - 5} заявок
                      </div>
                    )}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Success Rate Statistics */}
      <div className="bg-white shadow overflow-hidden sm:rounded-lg">
        <div className="px-4 py-5 sm:px-6">
          <h3 className="text-lg leading-6 font-medium text-gray-900">Показатели успешности</h3>
          <p className="mt-1 max-w-2xl text-sm text-gray-500">Общие показатели эффективности процесса трудоустройства</p>
        </div>
        <div className="border-t border-gray-200">
          <dl>
            <div className="bg-gray-50 dark:bg-gray-800 px-4 py-5 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-6">
              <dt className="text-sm font-medium text-gray-500">Общий процент успешности</dt>
              <dd className="mt-1 text-sm text-gray-900 font-medium sm:mt-0 sm:col-span-2">
                {analytics.success_rate.overall_success_rate.toFixed(1)}%
              </dd>
            </div>
            <div className="bg-white dark:bg-gray-700 px-4 py-5 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-6">
              <dt className="text-sm font-medium text-gray-500">Переход скрининг → интервью</dt>
              <dd className="mt-1 text-sm text-gray-900 font-medium sm:mt-0 sm:col-span-2">
                {analytics.success_rate.screening_to_interview_rate.toFixed(1)}%
              </dd>
            </div>
            <div className="bg-gray-50 dark:bg-gray-800 px-4 py-5 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-6">
              <dt className="text-sm font-medium text-gray-500">Успешность интервью</dt>
              <dd className="mt-1 text-sm text-gray-900 font-medium sm:mt-0 sm:col-span-2">
                {analytics.success_rate.interview_success_rate.toFixed(1)}%
              </dd>
            </div>
            <div className="bg-white dark:bg-gray-700 px-4 py-5 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-6">
              <dt className="text-sm font-medium text-gray-500">Заявки с URL вакансий</dt>
              <dd className="mt-1 text-sm text-gray-900 sm:mt-0 sm:col-span-2">
                {analytics.success_rate.applications_with_urls} из {analytics.total_applications} 
                <span className="ml-2 text-xs text-gray-500">
                  ({((analytics.success_rate.applications_with_urls / analytics.total_applications) * 100).toFixed(1)}%)
                </span>
              </dd>
            </div>
          </dl>
        </div>
      </div>

      {/* Response Time Statistics */}
      <div className="bg-white shadow overflow-hidden sm:rounded-lg">
        <div className="px-4 py-5 sm:px-6">
          <h3 className="text-lg leading-6 font-medium text-gray-900">Время отклика</h3>
          <p className="mt-1 max-w-2xl text-sm text-gray-500">Статистика по времени между этапами</p>
        </div>
        <div className="border-t border-gray-200">
          <dl>
            <div className="bg-gray-50 dark:bg-gray-800 px-4 py-5 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-6">
              <dt className="text-sm font-medium text-gray-500">Среднее время до скрининга</dt>
              <dd className="mt-1 text-sm text-gray-900 font-medium sm:mt-0 sm:col-span-2">
                {analytics.response_times.avg_days_to_screening.toFixed(1)} дней
              </dd>
            </div>
            <div className="bg-white dark:bg-gray-700 px-4 py-5 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-6">
              <dt className="text-sm font-medium text-gray-500">Среднее время до интервью</dt>
              <dd className="mt-1 text-sm text-gray-900 font-medium sm:mt-0 sm:col-span-2">
                {analytics.response_times.avg_days_to_interview.toFixed(1)} дней
              </dd>
            </div>
            <div className="bg-gray-50 dark:bg-gray-800 px-4 py-5 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-6">
              <dt className="text-sm font-medium text-gray-500">Самый быстрый скрининг</dt>
              <dd className="mt-1 text-sm text-green-600 font-medium sm:mt-0 sm:col-span-2">
                {analytics.response_times.fastest_screening_days} дней
              </dd>
            </div>
            <div className="bg-white dark:bg-gray-700 px-4 py-5 sm:grid sm:grid-cols-3 sm:gap-4 sm:px-6">
              <dt className="text-sm font-medium text-gray-500">Самый медленный скрининг</dt>
              <dd className="mt-1 text-sm text-red-600 font-medium sm:mt-0 sm:col-span-2">
                {analytics.response_times.slowest_screening_days} дней
              </dd>
            </div>
          </dl>
        </div>
      </div>

      {/* Top Performing Students */}
      {analytics.top_performing_students.length > 0 && (
        <div className="bg-white shadow overflow-hidden sm:rounded-lg">
          <div className="px-4 py-5 sm:px-6">
            <h3 className="text-lg leading-6 font-medium text-gray-900 dark:text-gray-100">Лучшие студенты</h3>
            <p className="mt-1 max-w-2xl text-sm text-gray-500 dark:text-gray-400">Студенты с наивысшими показателями успешности</p>
          </div>
          <div className="border-t border-gray-200 dark:border-gray-700">
            <ul className="divide-y divide-gray-200 dark:divide-gray-700">
              {analytics.top_performing_students.slice(0, 5).map((student, index) => (
                <li key={student.student_email} className="px-4 py-4 flex items-center justify-between">
                  <div className="flex items-center">
                    <div className="flex-shrink-0">
                      <div className={`h-8 w-8 rounded-full flex items-center justify-center text-sm font-medium ${
                        index === 0 ? 'bg-yellow-100 text-yellow-800' :
                        index === 1 ? 'bg-gray-100 text-gray-800' :
                        index === 2 ? 'bg-orange-100 text-orange-800' :
                        'bg-blue-100 text-blue-800'
                      }`}>
                        {index + 1}
                      </div>
                    </div>
                    <div className="ml-4">
                      <div className="text-sm font-medium text-gray-900 dark:text-gray-100">{student.student_name}</div>
                      <div className="text-sm text-gray-500 dark:text-gray-400">{student.student_email}</div>
                    </div>
                  </div>
                  <div className="text-right">
                    <div className="text-sm font-medium text-gray-900 dark:text-gray-100">{student.success_rate.toFixed(1)}% успешность</div>
                    <div className="text-sm text-gray-500 dark:text-gray-400">
                      {student.total_applications} заявок, {student.interviews_passed} интервью пройдено
                    </div>
                  </div>
                </li>
              ))}
            </ul>
          </div>
        </div>
      )}

      {/* Daily Activity Stats */}
      {analytics.daily_stats.length > 0 && (
        <div className="bg-white shadow overflow-hidden sm:rounded-lg">
          <div className="px-4 py-5 sm:px-6">
            <h3 className="text-lg leading-6 font-medium text-gray-900 dark:text-gray-100">Активность за последние 7 дней</h3>
            <p className="mt-1 max-w-2xl text-sm text-gray-500 dark:text-gray-400">Ежедневная статистика по заявкам, скринингам и интервью</p>
          </div>
          <div className="border-t border-gray-200 dark:border-gray-700">
            <div className="overflow-x-auto">
              <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                <thead className="bg-gray-50 dark:bg-gray-800">
                  <tr>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Дата</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Заявки</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Скрининги</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">Интервью</th>
                  </tr>
                </thead>
                <tbody className="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
                  {analytics.daily_stats.map((day) => (
                    <tr key={day.date}>
                      <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 dark:text-gray-100">
                        {new Date(day.date).toLocaleDateString('ru-RU')}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-gray-100">{day.applications_count}</td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-gray-100">{day.screenings_count}</td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-gray-100">{day.interviews_count}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      )}

    </div>
  );
}