// Консистентные цветовые схемы для всех статусов приложений

export const statusColors = {
  // Статусы заявок
  waiting: {
    light: 'bg-amber-100 text-amber-800 border-amber-200',
    dark: 'bg-amber-900/20 text-amber-400 border-amber-800',
    icon: 'text-amber-500'
  },
  rejected: {
    light: 'bg-red-100 text-red-800 border-red-200',
    dark: 'bg-red-900/20 text-red-400 border-red-800',
    icon: 'text-red-500'
  },
  next_stage: {
    light: 'bg-blue-100 text-blue-800 border-blue-200',
    dark: 'bg-blue-900/20 text-blue-400 border-blue-800',
    icon: 'text-blue-500'
  },
  ignored: {
    light: 'bg-gray-100 text-gray-800 border-gray-200',
    dark: 'bg-gray-800 text-gray-400 border-gray-700',
    icon: 'text-gray-500'
  },
  
  // Результаты скрининга/интервью
  passed: {
    light: 'bg-green-100 text-green-800 border-green-200',
    dark: 'bg-green-900/20 text-green-400 border-green-800',
    icon: 'text-green-500'
  },
  failed: {
    light: 'bg-red-100 text-red-800 border-red-200',
    dark: 'bg-red-900/20 text-red-400 border-red-800',
    icon: 'text-red-500'
  },
  pending: {
    light: 'bg-yellow-100 text-yellow-800 border-yellow-200',
    dark: 'bg-yellow-900/20 text-yellow-400 border-yellow-800',
    icon: 'text-yellow-500'
  }
} as const;

// Функция для получения классов статуса
export function getStatusClasses(status: string, theme: 'light' | 'dark' = 'light') {
  const statusKey = status as keyof typeof statusColors;
  return statusColors[statusKey]?.[theme] || statusColors.ignored[theme];
}

// Функция для получения цвета иконки статуса
export function getStatusIconColor(status: string) {
  const statusKey = status as keyof typeof statusColors;
  return statusColors[statusKey]?.icon || statusColors.ignored.icon;
}

// Переводы статусов
export const statusLabels = {
  waiting: 'Ожидание',
  rejected: 'Отклонена',
  next_stage: 'Следующий этап', 
  ignored: 'Игнорируется',
  passed: 'Пройден',
  failed: 'Провален',
  pending: 'В ожидании'
} as const;

// Функция для получения перевода статуса
export function getStatusLabel(status: string) {
  const statusKey = status as keyof typeof statusLabels;
  return statusLabels[statusKey] || status;
}

// Цвета для Activity Heatmap
export const heatmapColors = {
  light: {
    empty: 'bg-gray-100',
    level1: 'bg-green-100',
    level2: 'bg-green-200', 
    level3: 'bg-green-400',
    level4: 'bg-green-600'
  },
  dark: {
    empty: 'bg-gray-800',
    level1: 'bg-green-900/40',
    level2: 'bg-green-800/60',
    level3: 'bg-green-600/80', 
    level4: 'bg-green-500'
  }
};

// Функция для получения цвета heatmap
export function getHeatmapColor(intensity: number, theme: 'light' | 'dark' = 'light') {
  const colors = heatmapColors[theme];
  switch (intensity) {
    case 0: return colors.empty;
    case 1: return colors.level1;
    case 2: return colors.level2;
    case 3: return colors.level3;
    case 4: return colors.level4;
    default: return colors.empty;
  }
}