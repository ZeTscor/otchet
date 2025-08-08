import { useState, useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { applicationsApi, filesApi } from '../utils/api';
import type { Application, CreateApplicationRequest } from '../types';
import { useTheme } from '../contexts/ThemeContext';
import { getStatusClasses, getStatusLabel } from '../utils/colors';
import { 
  PlusIcon, 
  PencilIcon, 
  TrashIcon,
  DocumentIcon,
  CalendarIcon,
  BuildingOfficeIcon,
  XMarkIcon
} from '@heroicons/react/24/outline';
import { format } from 'date-fns';
import { Dialog } from '@headlessui/react';

export default function ApplicationsPage() {
  const { theme } = useTheme();
  const [applications, setApplications] = useState<Application[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string>('');
  const [showAddModal, setShowAddModal] = useState(false);
  const [editingApp, setEditingApp] = useState<Application | null>(null);
  const [uploadingScreening, setUploadingScreening] = useState<number | null>(null);
  const [uploadingInterview, setUploadingInterview] = useState<number | null>(null);

  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<CreateApplicationRequest>();

  useEffect(() => {
    fetchApplications();
  }, []);

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

  const onSubmit = async (data: CreateApplicationRequest) => {
    try {
      if (editingApp) {
        const updated = await applicationsApi.updateApplication(editingApp.id, data);
        setApplications(apps => apps.map(app => app.id === editingApp.id ? updated : app));
        setEditingApp(null);
      } else {
        const newApp = await applicationsApi.createApplication(data);
        setApplications(apps => [newApp, ...apps]);
        setShowAddModal(false);
      }
      reset();
    } catch (err: any) {
      setError('Ошибка сохранения заявки');
    }
  };

  const handleDelete = async (id: number) => {
    if (!confirm('Вы уверены, что хотите удалить эту заявку?')) return;
    
    try {
      await applicationsApi.deleteApplication(id);
      setApplications(apps => apps.filter(app => app.id !== id));
    } catch (err: any) {
      setError('Ошибка удаления заявки');
    }
  };

  const handleStatusUpdate = async (id: number, status: string) => {
    try {
      const updated = await applicationsApi.updateApplication(id, { status: status as any });
      setApplications(apps => apps.map(app => app.id === id ? updated : app));
    } catch (err: any) {
      setError('Ошибка обновления статуса');
    }
  };

  const handleFileUpload = async (appId: number, type: 'screening' | 'interview', file: File, data: any) => {
    const formData = new FormData();
    formData.append('file', file);
    if (data.date) formData.append(`${type}_date`, data.date);
    if (data.result) {
      const statusFieldName = type === 'screening' ? 'screening_status' : 'interview_status';
      formData.append(statusFieldName, data.result);
    }

    try {
      if (type === 'screening') {
        setUploadingScreening(appId);
        await applicationsApi.uploadScreening(appId, formData);
      } else {
        setUploadingInterview(appId);
        await applicationsApi.uploadInterview(appId, formData);
      }
      await fetchApplications(); // Refresh to get updated data
    } catch (err: any) {
      setError(`Ошибка загрузки файла ${type === 'screening' ? 'скрининга' : 'собеседования'}`);
    } finally {
      setUploadingScreening(null);
      setUploadingInterview(null);
    }
  };


  if (loading) {
    return <div className="flex justify-center items-center h-64">
      <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-indigo-500"></div>
    </div>;
  }

  return (
    <div className="space-y-6">
      <div className="sm:flex sm:items-center sm:justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900 dark:text-gray-100">Заявки на работу</h1>
          <p className="mt-2 text-sm text-gray-700 dark:text-gray-300">
            Управляйте своими заявками на работу и отслеживайте их прогресс
          </p>
        </div>
        <button
          onClick={() => setShowAddModal(true)}
          className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700"
        >
          <PlusIcon className="-ml-1 mr-2 h-5 w-5" />
          Добавить заявку
        </button>
      </div>

      {error && (
        <div className="bg-red-50 dark:bg-red-900/20 border border-red-300 dark:border-red-800 text-red-700 dark:text-red-400 px-4 py-3 rounded">
          {error}
        </div>
      )}

      <div className="bg-white dark:bg-gray-800 shadow overflow-hidden sm:rounded-md">
        {applications.length === 0 ? (
          <div className="text-center py-8">
            <BuildingOfficeIcon className="mx-auto h-12 w-12 text-gray-400" />
            <h3 className="mt-2 text-sm font-medium text-gray-900 dark:text-gray-100">Нет заявок</h3>
            <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">Начните с создания первой заявки.</p>
          </div>
        ) : (
          <ul className="divide-y divide-gray-200 dark:divide-gray-700">
            {applications.map((application) => (
              <li key={application.id} className="px-6 py-4">
                <div className="flex items-center justify-between">
                  <div className="flex items-center">
                    <BuildingOfficeIcon className="h-10 w-10 text-gray-400 dark:text-gray-500" />
                    <div className="ml-4">
                      <div className="flex items-center">
                        <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100">{application.company_name}</h3>
                        <span className={`ml-2 px-2.5 py-0.5 rounded-full text-xs font-medium ${getStatusClasses(application.status, theme)}`}>
                          {getStatusLabel(application.status)}
                        </span>
                      </div>
                      <div className="flex items-center text-sm text-gray-500 dark:text-gray-400 space-x-4">
                        <span className="flex items-center">
                          <CalendarIcon className="mr-1 h-4 w-4" />
                          Подана {format(new Date(application.application_date), 'dd.MM.yyyy')}
                        </span>
                        {application.job_url && (
                          <a 
                            href={application.job_url} 
                            target="_blank" 
                            rel="noopener noreferrer"
                            className="text-indigo-600 hover:text-indigo-500"
                          >
                            Посмотреть вакансию
                          </a>
                        )}
                      </div>
                    </div>
                  </div>
                  <div className="flex items-center space-x-2">
                    <select
                      value={application.status}
                      onChange={(e) => handleStatusUpdate(application.id, e.target.value)}
                      className="text-sm border border-gray-300 dark:border-gray-600 rounded-md px-2 py-1 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                    >
                      <option value="waiting">Ожидание</option>
                      <option value="next_stage">Следующий этап</option>
                      <option value="rejected">Отклонена</option>
                      <option value="ignored">Игнорируется</option>
                    </select>
                    <button
                      onClick={() => setEditingApp(application)}
                      className="text-indigo-600 hover:text-indigo-900"
                    >
                      <PencilIcon className="h-5 w-5" />
                    </button>
                    <button
                      onClick={() => handleDelete(application.id)}
                      className="text-red-600 hover:text-red-900"
                    >
                      <TrashIcon className="h-5 w-5" />
                    </button>
                  </div>
                </div>

                {/* File Upload Sections - Only show when status is next_stage */}
                {application.status === 'next_stage' && (
                  <div className="mt-4 grid grid-cols-1 md:grid-cols-2 gap-4">
                    {/* Screening Section */}
                    <div className="border border-gray-200 dark:border-gray-700 rounded-lg p-4 bg-white dark:bg-gray-800">
                      <h4 className="font-medium text-gray-900 dark:text-gray-100 mb-2">Скрининг</h4>
                      {application.screening ? (
                        <div className="space-y-2">
                          {application.screening.file_path && (
                            <a
                              href={filesApi.getFileUrl(application.screening.file_path)}
                              target="_blank"
                              rel="noopener noreferrer"
                              className="flex items-center text-sm text-indigo-600 hover:text-indigo-500"
                            >
                              <DocumentIcon className="h-4 w-4 mr-1" />
                              Посмотреть файл
                            </a>
                          )}
                          {application.screening.screening_date && (
                            <p className="text-sm text-gray-600 dark:text-gray-300">
                              Дата: {format(new Date(application.screening.screening_date), 'dd.MM.yyyy')}
                            </p>
                          )}
                          {application.screening.result && (
                            <p className="text-sm text-gray-600 dark:text-gray-300">
                              Статус: <span className={`inline-flex px-2 py-1 text-xs rounded-full ${
                                application.screening.result === 'passed' 
                                  ? 'bg-green-100 text-green-800' 
                                  : 'bg-red-100 text-red-800'
                              }`}>
                                {application.screening.result === 'passed' ? 'Пройден' : application.screening.result === 'failed' ? 'Провален' : application.screening.result}
                              </span>
                            </p>
                          )}
                          
                          {/* Возможность обновить статус скрининга */}
                          <div className="mt-2 pt-2 border-t border-gray-100">
                            <ScreeningUploadForm 
                              appId={application.id}
                              onUpload={handleFileUpload}
                              loading={uploadingScreening === application.id}
                              isUpdate={true}
                            />
                          </div>
                        </div>
                      ) : (
                        <ScreeningUploadForm 
                          appId={application.id}
                          onUpload={handleFileUpload}
                          loading={uploadingScreening === application.id}
                          isUpdate={false}
                        />
                      )}
                    </div>

                    {/* Interview Section */}
                    <div className="border border-gray-200 rounded-lg p-4">
                      <h4 className="font-medium text-gray-900 mb-2">Собеседование</h4>
                      {application.interview ? (
                        <div className="space-y-2">
                          {application.interview.file_path && (
                            <a
                              href={filesApi.getFileUrl(application.interview.file_path)}
                              target="_blank"
                              rel="noopener noreferrer"
                              className="flex items-center text-sm text-indigo-600 hover:text-indigo-500"
                            >
                              <DocumentIcon className="h-4 w-4 mr-1" />
                              Посмотреть файл
                            </a>
                          )}
                          {application.interview.interview_date && (
                            <p className="text-sm text-gray-600 dark:text-gray-300">
                              Дата: {format(new Date(application.interview.interview_date), 'dd.MM.yyyy')}
                            </p>
                          )}
                          {application.interview.result && (
                            <p className="text-sm text-gray-600 dark:text-gray-300">
                              Статус: <span className={`inline-flex px-2 py-1 text-xs rounded-full ${
                                application.interview.result === 'passed' 
                                  ? 'bg-green-100 text-green-800' 
                                  : 'bg-red-100 text-red-800'
                              }`}>
                                {application.interview.result === 'passed' ? 'Пройден' : application.interview.result === 'failed' ? 'Провален' : application.interview.result}
                              </span>
                            </p>
                          )}
                          
                          {/* Возможность обновить статус собеседования */}
                          <div className="mt-2 pt-2 border-t border-gray-100">
                            <InterviewUploadForm 
                              appId={application.id}
                              onUpload={handleFileUpload}
                              loading={uploadingInterview === application.id}
                              canUpload={true}
                              isUpdate={true}
                            />
                          </div>
                        </div>
                      ) : (
                        <InterviewUploadForm 
                          appId={application.id}
                          onUpload={handleFileUpload}
                          loading={uploadingInterview === application.id}
                          canUpload={application.screening?.result === 'passed'}
                          isUpdate={false}
                        />
                      )}
                    </div>
                  </div>
                )}
              </li>
            ))}
          </ul>
        )}
      </div>

      {/* Add/Edit Modal */}
      <Dialog 
        open={showAddModal || editingApp !== null} 
        onClose={() => {
          setShowAddModal(false);
          setEditingApp(null);
          reset();
        }}
        className="relative z-50"
      >
        <div className="fixed inset-0 bg-black/30" aria-hidden="true" />
        <div className="fixed inset-0 flex items-center justify-center p-4">
          <Dialog.Panel className="mx-auto max-w-sm rounded bg-white dark:bg-gray-800 p-6">
            <div className="flex justify-between items-center mb-4">
              <Dialog.Title className="text-lg font-medium text-gray-900 dark:text-gray-100">
                {editingApp ? 'Редактировать заявку' : 'Добавить заявку'}
              </Dialog.Title>
              <button 
                onClick={() => {
                  setShowAddModal(false);
                  setEditingApp(null);
                  reset();
                }}
                className="text-gray-400 hover:text-gray-600 dark:text-gray-500 dark:hover:text-gray-300"
              >
                <XMarkIcon className="h-5 w-5" />
              </button>
            </div>
            
            <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">Компания</label>
                <input
                  {...register('company_name', { required: 'Компания обязательна' })}
                  defaultValue={editingApp?.company_name}
                  className="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
                />
                {errors.company_name && <p className="text-red-600 text-sm">{errors.company_name.message}</p>}
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">URL вакансии (необязательно)</label>
                <input
                  {...register('job_url')}
                  defaultValue={editingApp?.job_url}
                  type="url"
                  className="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">Дата подачи</label>
                <input
                  {...register('application_date', { required: 'Дата подачи обязательна' })}
                  defaultValue={editingApp ? editingApp.application_date.split('T')[0] : ''}
                  type="date"
                  className="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
                />
                {errors.application_date && <p className="text-red-600 text-sm">{errors.application_date.message}</p>}
              </div>

              <div className="flex justify-end space-x-3">
                <button
                  type="button"
                  onClick={() => {
                    setShowAddModal(false);
                    setEditingApp(null);
                    reset();
                  }}
                  className="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 rounded-md"
                >
                  Отмена
                </button>
                <button
                  type="submit"
                  className="px-4 py-2 text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 rounded-md"
                >
                  {editingApp ? 'Обновить' : 'Создать'}
                </button>
              </div>
            </form>
          </Dialog.Panel>
        </div>
      </Dialog>
    </div>
  );
}

// Helper components for file uploads
function ScreeningUploadForm({ appId, onUpload, loading, isUpdate = false }: any) {
  const [file, setFile] = useState<File | null>(null);
  const [date, setDate] = useState('');
  const [result, setResult] = useState('');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    // Allow submission if file is provided OR date is provided (for scheduling)
    const canSubmit = isUpdate 
      ? (file !== null || date !== '' || result !== '') 
      : (file !== null || date !== '');
    
    if (canSubmit) {
      onUpload(appId, 'screening', file, { date, result });
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-2">
      {isUpdate ? (
        <p className="text-xs text-gray-500 dark:text-gray-400">Обновить скрининг (заполните только нужные поля):</p>
      ) : (
        <p className="text-xs text-gray-500 dark:text-gray-400">Загрузите файл или укажите дату для планирования скрининга:</p>
      )}
      <input
        type="file"
        accept="audio/*,video/*"
        onChange={(e) => setFile(e.target.files?.[0] || null)}
        className="block w-full text-sm text-gray-500 file:mr-4 file:py-2 file:px-4 file:rounded-full file:border-0 file:text-sm file:font-semibold file:bg-indigo-50 file:text-indigo-700 hover:file:bg-indigo-100"
      />
      <input
        type="date"
        value={date}
        onChange={(e) => setDate(e.target.value)}
        placeholder="Дата скрининга"
        className="block w-full px-3 py-2 text-sm border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
      />
      <select
        value={result}
        onChange={(e) => setResult(e.target.value)}
        className="block w-full px-3 py-2 text-sm border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
      >
        <option value="">Выберите результат</option>
        <option value="passed">Пройден</option>
        <option value="failed">Провален</option>
      </select>
      <button
        type="submit"
        disabled={loading || (isUpdate ? (file === null && date === '' && result === '') : (file === null && date === ''))}
        className="w-full px-3 py-2 text-sm bg-indigo-600 text-white rounded-md hover:bg-indigo-700 disabled:opacity-50"
      >
        {loading ? 'Загрузка...' : (isUpdate ? 'Обновить скрининг' : (file ? 'Загрузить скрининг' : 'Запланировать скрининг'))}
      </button>
    </form>
  );
}

function InterviewUploadForm({ appId, onUpload, loading, canUpload, isUpdate = false }: any) {
  const [file, setFile] = useState<File | null>(null);
  const [date, setDate] = useState('');
  const [result, setResult] = useState('');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    // For new uploads, require a file. For updates, allow any field to be changed
    const canSubmit = isUpdate 
      ? (file !== null || date !== '' || result !== '') 
      : (file !== null && canUpload);
    
    if (canSubmit) {
      onUpload(appId, 'interview', file, { date, result });
    }
  };

  if (!canUpload && !isUpdate) {
    return (
      <div className="text-sm text-gray-500 dark:text-gray-400">
        Загрузка собеседования доступна после прохождения скрининга
      </div>
    );
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-2">
      {isUpdate && <p className="text-xs text-gray-500 dark:text-gray-400">Обновить собеседование (заполните только нужные поля):</p>}
      <input
        type="file"
        accept="audio/*,video/*"
        onChange={(e) => setFile(e.target.files?.[0] || null)}
        className="block w-full text-sm text-gray-500 file:mr-4 file:py-2 file:px-4 file:rounded-full file:border-0 file:text-sm file:font-semibold file:bg-indigo-50 file:text-indigo-700 hover:file:bg-indigo-100"
        required={!isUpdate}
      />
      <input
        type="date"
        value={date}
        onChange={(e) => setDate(e.target.value)}
        placeholder="Дата собеседования"
        className="block w-full px-3 py-2 text-sm border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
      />
      <select
        value={result}
        onChange={(e) => setResult(e.target.value)}
        className="block w-full px-3 py-2 text-sm border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
      >
        <option value="">Выберите результат</option>
        <option value="passed">Пройден</option>
        <option value="failed">Провален</option>
      </select>
      <button
        type="submit"
        disabled={loading || (isUpdate ? (file === null && date === '' && result === '') : file === null)}
        className="w-full px-3 py-2 text-sm bg-indigo-600 text-white rounded-md hover:bg-indigo-700 disabled:opacity-50"
      >
        {loading ? 'Загрузка...' : (isUpdate ? 'Обновить собеседование' : 'Загрузить собеседование')}
      </button>
    </form>
  );
}