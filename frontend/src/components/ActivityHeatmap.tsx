import { useState } from 'react';
import { useTheme } from '../contexts/ThemeContext';
import { getHeatmapColor } from '../utils/colors';
import type { ActivityData } from '../types';

interface ActivityHeatmapProps {
  data: ActivityData[];
  loading?: boolean;
}

export default function ActivityHeatmap({ data, loading }: ActivityHeatmapProps) {
  const { theme } = useTheme();
  const [hoveredCell, setHoveredCell] = useState<{ date: string; activity: number } | null>(null);

  // Get the max activity for scaling colors
  const maxActivity = Math.max(...data.map(d => d.total_activity), 1);

  // Get activity intensity (0-4 scale like GitHub)
  const getIntensity = (activity: number): number => {
    if (activity === 0) return 0;
    const ratio = activity / maxActivity;
    if (ratio <= 0.25) return 1;
    if (ratio <= 0.5) return 2;
    if (ratio <= 0.75) return 3;
    return 4;
  };

  // Get color class based on intensity using theme
  const getColorClass = (intensity: number): string => {
    return getHeatmapColor(intensity, theme);
  };

  // Group data by weeks for display
  const groupDataByWeeks = (data: ActivityData[]) => {
    const weeks: ActivityData[][] = [];
    let currentWeek: ActivityData[] = [];
    
    data.forEach((day, index) => {
      const dayOfWeek = new Date(day.date).getDay();
      
      if (index === 0) {
        // Add empty cells for the first week to align with Sunday
        for (let i = 0; i < dayOfWeek; i++) {
          currentWeek.push({
            date: '',
            applications_count: 0,
            screenings_count: 0,
            interviews_count: 0,
            total_activity: 0,
          });
        }
      }
      
      currentWeek.push(day);
      
      if (dayOfWeek === 6 || index === data.length - 1) {
        // End of week or last day
        weeks.push([...currentWeek]);
        currentWeek = [];
      }
    });
    
    return weeks;
  };

  // Get month labels positioned correctly for each week column
  const getMonthLabelsForWeeks = (weeks: ActivityData[][]) => {
    const monthLabels: { [key: number]: string } = {};
    const monthNames = ['Янв', 'Фев', 'Мар', 'Апр', 'Май', 'Июн', 'Июл', 'Авг', 'Сен', 'Окт', 'Ноя', 'Дек'];
    let lastMonth = -1;

    weeks.forEach((week, weekIndex) => {
      // Get the first valid date in this week (usually Monday or first day)
      const firstDayWithDate = week.find(day => day.date);
      if (firstDayWithDate) {
        const currentMonth = new Date(firstDayWithDate.date).getMonth();
        
        // If this is a new month and it's not the first week, add the label
        if (currentMonth !== lastMonth) {
          monthLabels[weekIndex] = monthNames[currentMonth];
          lastMonth = currentMonth;
        }
      }
    });

    return monthLabels;
  };

  const weeks = groupDataByWeeks(data);
  const dayLabels = ['Вс', 'Пн', 'Вт', 'Ср', 'Чт', 'Пт', 'Сб'];
  const monthLabelsMap = getMonthLabelsForWeeks(weeks);

  if (loading) {
    return (
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6 transition-colors">
        <div className="animate-pulse">
          <div className="h-4 bg-gray-200 dark:bg-gray-700 rounded w-48 mb-4"></div>
          <div className="grid grid-cols-53 gap-1">
            {Array.from({ length: 371 }).map((_, i) => (
              <div key={i} className="h-3 w-3 bg-gray-200 dark:bg-gray-700 rounded-sm"></div>
            ))}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6 transition-colors">
      <div className="mb-4">
        <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100">Карта активности</h3>
        <p className="text-sm text-gray-500 dark:text-gray-400">Ваша активность за последний год</p>
      </div>
      
      <div className="relative overflow-x-auto">
        <div className="inline-block min-w-full">
          {/* Month labels */}
          <div className="flex mb-2">
            <div className="w-8"></div>
            <div className="flex text-xs text-gray-500 dark:text-gray-400" style={{ gap: '4px' }}>
              {weeks.map((_, weekIndex) => (
                <div key={weekIndex} className="w-3 text-center">
                  {monthLabelsMap[weekIndex] || ''}
                </div>
              ))}
            </div>
          </div>
          
          <div className="flex">
            {/* Day labels */}
            <div className="flex flex-col justify-between text-xs text-gray-500 dark:text-gray-400 mr-2">
              {dayLabels.map((day, index) => (
                <span key={index} className="h-3 flex items-center" style={{ lineHeight: '12px' }}>
                  {index % 2 === 1 ? day : ''}
                </span>
              ))}
            </div>
            
            {/* Heatmap grid */}
            <div className="grid grid-flow-col gap-1" style={{ gridTemplateRows: 'repeat(7, 12px)' }}>
              {weeks.map((week, weekIndex) => (
                week.map((day, dayIndex) => (
                  <div
                    key={`${weekIndex}-${dayIndex}`}
                    className={`w-3 h-3 rounded-sm cursor-pointer transition-all duration-200 ${
                      day.date ? getColorClass(getIntensity(day.total_activity)) : 'bg-transparent'
                    } ${hoveredCell?.date === day.date ? 'ring-2 ring-gray-400' : ''}`}
                    title={
                      day.date 
                        ? `${new Date(day.date).toLocaleDateString('ru-RU')}: ${day.total_activity} активности`
                        : ''
                    }
                    onMouseEnter={() => day.date && setHoveredCell({ date: day.date, activity: day.total_activity })}
                    onMouseLeave={() => setHoveredCell(null)}
                  />
                ))
              ))}
            </div>
          </div>
          
          {/* Legend */}
          <div className="flex items-center justify-between mt-4">
            <span className="text-xs text-gray-500 dark:text-gray-400">Меньше</span>
            <div className="flex items-center space-x-1">
              {[0, 1, 2, 3, 4].map((intensity) => (
                <div
                  key={intensity}
                  className={`w-3 h-3 rounded-sm ${getColorClass(intensity)}`}
                />
              ))}
            </div>
            <span className="text-xs text-gray-500 dark:text-gray-400">Больше</span>
          </div>
          
          {/* Tooltip */}
          {hoveredCell && (
            <div className="mt-2 text-sm text-gray-600 dark:text-gray-300">
              <strong>{new Date(hoveredCell.date).toLocaleDateString('ru-RU')}</strong>: {hoveredCell.activity} активности
            </div>
          )}
        </div>
      </div>
    </div>
  );
}