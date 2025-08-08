import { useState, useEffect } from 'react';
import { adminApi, filesApi } from '../utils/api';
import type { Application, ActivityData } from '../types';
import ActivityHeatmap from '../components/ActivityHeatmap';
import { useTheme } from '../contexts/ThemeContext';
import { getStatusClasses, getStatusLabel } from '../utils/colors';
import { 
  UserIcon,
  EyeIcon,
  CalendarIcon,
  BuildingOfficeIcon,
  ChevronDownIcon,
  ChevronRightIcon
} from '@heroicons/react/24/outline';
import { format } from 'date-fns';

interface StudentWithApplications {
  user_id: number;
  email: string;
  first_name: string;
  last_name: string;
  applications: Application[];
}

export default function StudentsPage() {
  const [students, setStudents] = useState<StudentWithApplications[]>([]);
  const [studentActivityData, setStudentActivityData] = useState<Map<number, ActivityData[]>>(new Map());
  const [loading, setLoading] = useState(true);
  const [activityLoadingMap, setActivityLoadingMap] = useState<Map<number, boolean>>(new Map());
  const [error, setError] = useState<string>('');
  const [searchTerm, setSearchTerm] = useState('');
  const [expandedStudents, setExpandedStudents] = useState<Set<number>>(new Set());

  useEffect(() => {
    fetchStudents();
  }, []);

  const toggleStudent = async (studentId: number) => {
    const newExpanded = new Set(expandedStudents);
    if (newExpanded.has(studentId)) {
      newExpanded.delete(studentId);
    } else {
      newExpanded.add(studentId);
      // Load activity data for this student if not already loaded
      if (!studentActivityData.has(studentId)) {
        await fetchStudentActivity(studentId);
      }
    }
    setExpandedStudents(newExpanded);
  };

  const fetchStudents = async () => {
    try {
      // Получаем все заявки и всех студентов параллельно
      const [allApplications, allStudents] = await Promise.all([
        adminApi.getAllApplications(),
        adminApi.getAllStudents()
      ]);
      
      // Создаем карту студентов для быстрого поиска
      const studentsMap = new Map<number, { email: string; first_name: string; last_name: string }>();
      allStudents.forEach(student => {
        studentsMap.set(student.id, {
          email: student.email,
          first_name: student.first_name,
          last_name: student.last_name
        });
      });
      
      // Группируем заявки по студентам
      const studentsWithAppsMap = new Map<number, StudentWithApplications>();
      
      allApplications.forEach((app: Application) => {
        if (!studentsWithAppsMap.has(app.user_id)) {
          const studentInfo = studentsMap.get(app.user_id);
          studentsWithAppsMap.set(app.user_id, {
            user_id: app.user_id,
            email: studentInfo?.email || `email@unknown.com`,
            first_name: studentInfo?.first_name || 'Имя',
            last_name: studentInfo?.last_name || 'Фамилия',
            applications: []
          });
        }
        studentsWithAppsMap.get(app.user_id)!.applications.push(app);
      });

      setStudents(Array.from(studentsWithAppsMap.values()));
    } catch (err: any) {
      setError('Ошибка загрузки данных студентов');
    } finally {
      setLoading(false);
    }
  };

  const fetchStudentActivity = async (studentId: number) => {
    try {
      setActivityLoadingMap(prev => new Map(prev).set(studentId, true));
      const data = await adminApi.getUserActivity(studentId);
      setStudentActivityData(prev => new Map(prev).set(studentId, data));
    } catch (err: any) {
      console.error(`Ошибка загрузки активности студента ${studentId}:`, err);
    } finally {
      setActivityLoadingMap(prev => new Map(prev).set(studentId, false));
    }
  };

  const { theme } = useTheme();

  const filteredStudents = students.filter(student =>
    student.email.toLowerCase().includes(searchTerm.toLowerCase()) ||
    student.first_name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    student.last_name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    `${student.first_name} ${student.last_name}`.toLowerCase().includes(searchTerm.toLowerCase()) ||
    student.applications.some(app => 
      app.company_name.toLowerCase().includes(searchTerm.toLowerCase())
    )
  );

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
      <div>
        <h1 className="text-2xl font-bold text-gray-900 dark:text-gray-100">Студенты и их заявки</h1>
        <p className="mt-2 text-sm text-gray-700 dark:text-gray-300">
          Полный список студентов с их заявками, скринингами и интервью
        </p>
      </div>

      {/* Search */}
      <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-4">
        <div className="max-w-md">
          <input
            type="text"
            placeholder="Поиск по имени, фамилии, email или компании..."
            className="block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 placeholder-gray-500 dark:placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
          />
        </div>
      </div>


      {/* Students List */}
      <div className="bg-white dark:bg-gray-800 shadow overflow-hidden sm:rounded-md">
        {filteredStudents.length === 0 ? (
          <div className="text-center py-8">
            <UserIcon className="mx-auto h-12 w-12 text-gray-400" />
            <h3 className="mt-2 text-sm font-medium text-gray-900 dark:text-gray-100">Студенты не найдены</h3>
            <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">Попробуйте изменить поисковый запрос.</p>
          </div>
        ) : (
          <ul className="divide-y divide-gray-200 dark:divide-gray-700">
            {filteredStudents.map((student) => {
              const isExpanded = expandedStudents.has(student.user_id);
              const screeningsPassed = student.applications.filter(app => app.screening?.result === 'passed').length;
              const interviewsPassed = student.applications.filter(app => app.interview?.result === 'passed').length;
              
              return (
                <li key={student.user_id} className="px-6 py-4">
                  <div 
                    className="flex items-center justify-between cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700 -mx-6 px-6 py-2 rounded transition-colors"
                    onClick={() => toggleStudent(student.user_id)}
                  >
                    <div className="flex items-center min-w-0">
                      <div className="flex-shrink-0">
                        {isExpanded ? (
                          <ChevronDownIcon className="h-5 w-5 text-gray-400 dark:text-gray-500" />
                        ) : (
                          <ChevronRightIcon className="h-5 w-5 text-gray-400 dark:text-gray-500" />
                        )}
                      </div>
                      <UserIcon className="h-8 w-8 text-gray-400 dark:text-gray-500 ml-3" />
                      <div className="ml-4 flex-1 min-w-0">
                        <div className="flex items-center space-x-4">
                          <div>
                            <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100 truncate">
                              {student.first_name} {student.last_name}
                            </h3>
                            <p className="text-sm text-gray-500 dark:text-gray-400 truncate">{student.email}</p>
                          </div>
                          <div className="flex items-center space-x-6 text-sm text-gray-600 dark:text-gray-400">
                            <div className="text-center">
                              <div className="font-medium text-gray-900 dark:text-gray-100">{student.applications.length}</div>
                              <div className="text-xs text-gray-500 dark:text-gray-400">заявок</div>
                            </div>
                            <div className="text-center">
                              <div className="font-medium text-green-600 dark:text-green-400">{screeningsPassed}</div>
                              <div className="text-xs text-gray-500 dark:text-gray-400">скрининг</div>
                            </div>
                            <div className="text-center">
                              <div className="font-medium text-blue-600 dark:text-blue-400">{interviewsPassed}</div>
                              <div className="text-xs text-gray-500 dark:text-gray-400">интервью</div>
                            </div>
                            {student.applications.length > 0 && (
                              <div className="text-center">
                                <div className="font-medium text-purple-600 dark:text-purple-400">
                                  {Math.round((interviewsPassed / student.applications.length) * 100)}%
                                </div>
                                <div className="text-xs text-gray-500 dark:text-gray-400">успех</div>
                              </div>
                            )}
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>

                  {/* Expanded Applications */}
                  {isExpanded && (
                    <div className="mt-4 ml-8 space-y-4 border-l-2 border-gray-200 dark:border-gray-700 pl-4">
                      {/* Individual Student Activity Heatmap */}
                      <div className="mb-6">
                        <h4 className="text-lg font-medium text-gray-900 dark:text-gray-100 mb-3">
                          Активность студента {student.first_name} {student.last_name}
                        </h4>
                        {studentActivityData.has(student.user_id) ? (
                          <ActivityHeatmap 
                            data={studentActivityData.get(student.user_id) || []} 
                            loading={activityLoadingMap.get(student.user_id) || false}
                          />
                        ) : (
                          <div className="bg-gray-50 dark:bg-gray-800 border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-lg p-4 text-center">
                            <p className="text-sm text-gray-500 dark:text-gray-400">Карта активности загружается...</p>
                          </div>
                        )}
                      </div>
                  {student.applications.map((application) => (
                    <div key={application.id} className="border border-gray-200 dark:border-gray-700 rounded-lg p-4 bg-white dark:bg-gray-800">
                      <div className="flex items-center justify-between mb-3">
                        <div className="flex items-center">
                          <BuildingOfficeIcon className="h-5 w-5 text-gray-400 dark:text-gray-500 mr-2" />
                          <span className="font-medium text-gray-900 dark:text-gray-100">{application.company_name}</span>
                          <span className={`ml-2 px-2.5 py-0.5 rounded-full text-xs font-medium ${getStatusClasses(application.status, theme)}`}>
                            {getStatusLabel(application.status)}
                          </span>
                        </div>
                        <div className="flex items-center text-sm text-gray-500 dark:text-gray-400">
                          <CalendarIcon className="h-4 w-4 mr-1" />
                          {format(new Date(application.application_date), 'dd.MM.yyyy')}
                        </div>
                      </div>

                      {/* Screening and Interview Info */}
                      <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mt-4">
                        {/* Screening */}
                        <div className="border border-gray-100 dark:border-gray-700 rounded p-3 bg-gray-50 dark:bg-gray-800">
                          <h4 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Скрининг</h4>
                          {application.screening ? (
                            <div className="space-y-2">
                              {application.screening.file_path && (
                                <div className="flex items-center justify-between">
                                  <span className="text-xs text-gray-500 dark:text-gray-400">Файл загружен</span>
                                  <a
                                    href={filesApi.getFileUrl(application.screening.file_path)}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    className="flex items-center text-xs text-indigo-600 hover:text-indigo-500"
                                  >
                                    <EyeIcon className="h-3 w-3 mr-1" />
                                    Открыть
                                  </a>
                                </div>
                              )}
                              {application.screening.screening_date && (
                                <p className="text-xs text-gray-500 dark:text-gray-400">
                                  Дата: {format(new Date(application.screening.screening_date), 'dd.MM.yyyy')}
                                </p>
                              )}
                              {application.screening.result && (
                                <span className={`inline-flex px-2 py-1 text-xs rounded-full ${
                                  application.screening.result === 'passed' 
                                    ? 'bg-green-100 text-green-800' 
                                    : 'bg-red-100 text-red-800'
                                }`}>
                                  {application.screening.result === 'passed' ? 'Пройден' : 'Провален'}
                                </span>
                              )}
                            </div>
                          ) : (
                            <span className="text-xs text-gray-400 dark:text-gray-500">Не загружен</span>
                          )}
                        </div>

                        {/* Interview */}
                        <div className="border border-gray-100 dark:border-gray-700 rounded p-3 bg-gray-50 dark:bg-gray-800">
                          <h4 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Интервью</h4>
                          {application.interview ? (
                            <div className="space-y-2">
                              {application.interview.file_path && (
                                <div className="flex items-center justify-between">
                                  <span className="text-xs text-gray-500 dark:text-gray-400">Файл загружен</span>
                                  <a
                                    href={filesApi.getFileUrl(application.interview.file_path)}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    className="flex items-center text-xs text-indigo-600 hover:text-indigo-500"
                                  >
                                    <EyeIcon className="h-3 w-3 mr-1" />
                                    Открыть
                                  </a>
                                </div>
                              )}
                              {application.interview.interview_date && (
                                <p className="text-xs text-gray-500 dark:text-gray-400">
                                  Дата: {format(new Date(application.interview.interview_date), 'dd.MM.yyyy')}
                                </p>
                              )}
                              {application.interview.result && (
                                <span className={`inline-flex px-2 py-1 text-xs rounded-full ${
                                  application.interview.result === 'passed' 
                                    ? 'bg-green-100 text-green-800' 
                                    : 'bg-red-100 text-red-800'
                                }`}>
                                  {application.interview.result === 'passed' ? 'Пройден' : 'Провален'}
                                </span>
                              )}
                            </div>
                          ) : (
                            <span className="text-xs text-gray-400 dark:text-gray-500">
                              {application.screening?.result === 'passed' ? 'Доступно для загрузки' : 'Недоступно'}
                            </span>
                          )}
                        </div>
                      </div>

                      {application.job_url && (
                        <div className="mt-3 pt-3 border-t border-gray-100 dark:border-gray-700">
                          <a 
                            href={application.job_url} 
                            target="_blank" 
                            rel="noopener noreferrer"
                            className="text-xs text-indigo-600 hover:text-indigo-500"
                          >
                            Посмотреть вакансию →
                          </a>
                        </div>
                      )}
                    </div>
                      ))}
                    </div>
                  )}
                </li>
              );
            })}
          </ul>
        )}
      </div>
    </div>
  );
}