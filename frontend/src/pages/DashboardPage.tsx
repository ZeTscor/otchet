import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { applicationsApi } from '../utils/api';
import type { Application, ActivityData } from '../types';
import ActivityHeatmap from '../components/ActivityHeatmap';
import { useAuthStore } from '../store/authStore';
import { useTheme } from '../contexts/ThemeContext';
import { getStatusClasses, getStatusLabel } from '../utils/colors';
import { 
  ClipboardDocumentListIcon, 
  ExclamationTriangleIcon,
  PlusIcon,
  CalendarIcon,
  BuildingOfficeIcon
} from '@heroicons/react/24/outline';
import { format, differenceInDays } from 'date-fns';

export default function DashboardPage() {
  const { user } = useAuthStore();
  const { theme } = useTheme();
  const [applications, setApplications] = useState<Application[]>([]);
  const [activityData, setActivityData] = useState<ActivityData[]>([]);
  const [loading, setLoading] = useState(true);
  const [activityLoading, setActivityLoading] = useState(true);
  const [error, setError] = useState<string>('');

  useEffect(() => {
    fetchApplications();
    // Only fetch activity for students, not admins
    if (user?.role === 'student') {
      fetchActivity();
    }
  }, [user]);

  const fetchApplications = async () => {
    try {
      const data = await applicationsApi.getApplications();
      setApplications(data);
    } catch (err: any) {
      setError('Ошибка загрузки заявок');
    } finally {
      setLoading(false);
    }
  };

  const fetchActivity = async () => {
    try {
      const data = await applicationsApi.getActivity();
      setActivityData(data);
    } catch (err: any) {
      console.error('Ошибка загрузки активности:', err);
    } finally {
      setActivityLoading(false);
    }
  };


  const getStaleApplications = () => {
    return applications.filter(app => {
      const daysSinceUpdate = differenceInDays(new Date(), new Date(app.updated_at));
      return daysSinceUpdate > 7 && ['waiting', 'next_stage'].includes(app.status);
    });
  };

  const getRecentApplications = () => {
    return applications
      .sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime())
      .slice(0, 5);
  };

  const stats = {
    total: applications.length,
    waiting: applications.filter(a => a.status === 'waiting').length,
    nextStage: applications.filter(a => a.status === 'next_stage').length,
    rejected: applications.filter(a => a.status === 'rejected').length,
    ignored: applications.filter(a => a.status === 'ignored').length,
  };

  const staleApplications = getStaleApplications();
  const recentApplications = getRecentApplications();

  if (loading) {
    return (
      <div className="flex justify-center items-center h-64">
        <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-indigo-500"></div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-50 border border-red-300 text-red-700 px-4 py-3 rounded">
        {error}
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="sm:flex sm:items-center sm:justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900 dark:text-gray-100">Панель управления</h1>
          <p className="mt-2 text-sm text-gray-700 dark:text-gray-300">
            Track your job applications and their progress
          </p>
        </div>
        <div className="mt-4 sm:mt-0">
          <Link
            to="/applications"
            className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
          >
            <PlusIcon className="-ml-1 mr-2 h-5 w-5" />
            Add Application
          </Link>
        </div>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-5">
        <div className="bg-white dark:bg-gray-800 overflow-hidden shadow rounded-lg">
          <div className="p-5">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <ClipboardDocumentListIcon className="h-6 w-6 text-gray-400" />
              </div>
              <div className="ml-5 w-0 flex-1">
                <dl>
                  <dt className="text-sm font-medium text-gray-500 dark:text-gray-400 truncate">Total</dt>
                  <dd className="text-lg font-medium text-gray-900 dark:text-gray-100">{stats.total}</dd>
                </dl>
              </div>
            </div>
          </div>
        </div>

        <div className="bg-white dark:bg-gray-800 overflow-hidden shadow rounded-lg">
          <div className="p-5">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <div className="w-6 h-6 bg-yellow-100 rounded-full flex items-center justify-center">
                  <div className="w-3 h-3 bg-yellow-400 rounded-full"></div>
                </div>
              </div>
              <div className="ml-5 w-0 flex-1">
                <dl>
                  <dt className="text-sm font-medium text-gray-500 dark:text-gray-400 truncate">Waiting</dt>
                  <dd className="text-lg font-medium text-gray-900 dark:text-gray-100">{stats.waiting}</dd>
                </dl>
              </div>
            </div>
          </div>
        </div>

        <div className="bg-white dark:bg-gray-800 overflow-hidden shadow rounded-lg">
          <div className="p-5">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <div className="w-6 h-6 bg-blue-100 rounded-full flex items-center justify-center">
                  <div className="w-3 h-3 bg-blue-400 rounded-full"></div>
                </div>
              </div>
              <div className="ml-5 w-0 flex-1">
                <dl>
                  <dt className="text-sm font-medium text-gray-500 dark:text-gray-400 truncate">Next Stage</dt>
                  <dd className="text-lg font-medium text-gray-900 dark:text-gray-100">{stats.nextStage}</dd>
                </dl>
              </div>
            </div>
          </div>
        </div>

        <div className="bg-white dark:bg-gray-800 overflow-hidden shadow rounded-lg">
          <div className="p-5">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <div className="w-6 h-6 bg-red-100 rounded-full flex items-center justify-center">
                  <div className="w-3 h-3 bg-red-400 rounded-full"></div>
                </div>
              </div>
              <div className="ml-5 w-0 flex-1">
                <dl>
                  <dt className="text-sm font-medium text-gray-500 dark:text-gray-400 truncate">Rejected</dt>
                  <dd className="text-lg font-medium text-gray-900 dark:text-gray-100">{stats.rejected}</dd>
                </dl>
              </div>
            </div>
          </div>
        </div>

        <div className="bg-white dark:bg-gray-800 overflow-hidden shadow rounded-lg">
          <div className="p-5">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <div className="w-6 h-6 bg-gray-100 rounded-full flex items-center justify-center">
                  <div className="w-3 h-3 bg-gray-400 rounded-full"></div>
                </div>
              </div>
              <div className="ml-5 w-0 flex-1">
                <dl>
                  <dt className="text-sm font-medium text-gray-500 dark:text-gray-400 truncate">Ignored</dt>
                  <dd className="text-lg font-medium text-gray-900 dark:text-gray-100">{stats.ignored}</dd>
                </dl>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Stale Applications Alert */}
      {staleApplications.length > 0 && (
        <div className="bg-yellow-50 dark:bg-yellow-900/20 border-l-4 border-yellow-400 dark:border-yellow-600 p-4">
          <div className="flex">
            <div className="flex-shrink-0">
              <ExclamationTriangleIcon className="h-5 w-5 text-yellow-400" />
            </div>
            <div className="ml-3">
              <p className="text-sm text-yellow-700 dark:text-yellow-400">
                У вас <strong>{staleApplications.length}</strong> заявок без обновлений более 7 дней.
                <Link to="/applications" className="font-medium underline hover:text-yellow-600 dark:hover:text-yellow-300 ml-1">
                  Review them here
                </Link>
              </p>
            </div>
          </div>
        </div>
      )}

      {/* Activity Heatmap - только для студентов */}
      {user?.role === 'student' && (
        <ActivityHeatmap data={activityData} loading={activityLoading} />
      )}

      {/* Recent Applications */}
      <div className="bg-white shadow overflow-hidden sm:rounded-md">
        <div className="px-4 py-5 sm:px-6">
          <h3 className="text-lg leading-6 font-medium text-gray-900 dark:text-gray-100">Последние заявки</h3>
          <p className="mt-1 max-w-2xl text-sm text-gray-500 dark:text-gray-400">
            Your latest job applications
          </p>
        </div>
        {recentApplications.length === 0 ? (
          <div className="text-center py-8">
            <ClipboardDocumentListIcon className="mx-auto h-12 w-12 text-gray-400" />
            <h3 className="mt-2 text-sm font-medium text-gray-900 dark:text-gray-100">No applications</h3>
            <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">Get started by creating your first application.</p>
            <div className="mt-6">
              <Link
                to="/applications"
                className="inline-flex items-center px-4 py-2 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
              >
                <PlusIcon className="-ml-1 mr-2 h-5 w-5" />
                Add Application
              </Link>
            </div>
          </div>
        ) : (
          <ul className="divide-y divide-gray-200 dark:divide-gray-700">
            {recentApplications.map((application) => (
              <li key={application.id}>
                <div className="px-4 py-4 flex items-center justify-between">
                  <div className="flex items-center">
                    <div className="flex-shrink-0">
                      <BuildingOfficeIcon className="h-10 w-10 text-gray-400 dark:text-gray-500" />
                    </div>
                    <div className="ml-4">
                      <div className="flex items-center">
                        <p className="text-sm font-medium text-gray-900 dark:text-gray-100 truncate">
                          {application.company_name}
                        </p>
                        <span className={`ml-2 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getStatusClasses(application.status, theme)}`}>
                          {getStatusLabel(application.status)}
                        </span>
                      </div>
                      <div className="flex items-center text-sm text-gray-500 dark:text-gray-400">
                        <CalendarIcon className="flex-shrink-0 mr-1.5 h-4 w-4" />
                        Подана {format(new Date(application.application_date), 'dd.MM.yyyy')}
                      </div>
                    </div>
                  </div>
                  <div className="flex items-center space-x-4">
                    {application.screening && (
                      <span className="text-xs text-blue-600 dark:text-blue-400 bg-blue-50 dark:bg-blue-900/20 px-2 py-1 rounded">
                        Скрининг: {application.screening.result === 'passed' ? 'Пройден' : application.screening.result === 'failed' ? 'Провален' : 'Ожидание'}
                      </span>
                    )}
                    {application.interview && (
                      <span className="text-xs text-green-600 dark:text-green-400 bg-green-50 dark:bg-green-900/20 px-2 py-1 rounded">
                        Собеседование: {application.interview.result === 'passed' ? 'Пройден' : application.interview.result === 'failed' ? 'Провален' : 'Ожидание'}
                      </span>
                    )}
                  </div>
                </div>
              </li>
            ))}
          </ul>
        )}
        {recentApplications.length > 0 && (
          <div className="bg-gray-50 dark:bg-gray-800 px-4 py-3 text-right">
            <Link
              to="/applications"
              className="text-sm font-medium text-indigo-600 dark:text-indigo-400 hover:text-indigo-500 dark:hover:text-indigo-300"
            >
              View all applications →
            </Link>
          </div>
        )}
      </div>
    </div>
  );
}