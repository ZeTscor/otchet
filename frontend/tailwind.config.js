/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'class', // Включаем поддержку темной темы через класс
  safelist: [
    // Принудительно включаем все dark mode классы
    { pattern: /^dark:/ },
    'dark',
    'dark:bg-gray-800',
    'dark:bg-gray-900',
    'dark:bg-gray-950',
    'dark:bg-gray-700',
    'dark:text-gray-100',
    'dark:text-gray-200',
    'dark:text-gray-300',
    'dark:text-gray-400',
    'dark:text-gray-500',
    'dark:border-gray-600',
    'dark:border-gray-700',
    'dark:border-gray-800',
    'dark:hover:bg-gray-700',
    'dark:hover:bg-gray-800',
    'dark:hover:text-gray-200',
    'dark:hover:text-gray-300',
    'dark:divide-gray-700',
    'dark:placeholder-gray-400',
    'dark:text-red-400',
    'dark:text-green-400',
    'dark:text-blue-400',
    'dark:text-yellow-400',
    'dark:text-indigo-400',
    'dark:text-purple-400',
    'dark:bg-red-900/20',
    'dark:bg-yellow-900/20',
    'dark:bg-blue-900/20',
    'dark:bg-green-900/20',
    'dark:border-red-800',
    'dark:border-yellow-600',
    'dark:border-yellow-800',
  ],
  theme: {
    extend: {
      colors: {
        // Дополнительные цвета для темной темы
        gray: {
          850: '#1f2937', // Между gray-800 и gray-900
          950: '#0f172a'  // Очень темный для фонов
        }
      }
    },
  },
  plugins: [],
}