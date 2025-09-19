import typography from '@tailwindcss/typography'

/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      fontFamily: {
        sans: ['Inter', 'ui-sans-serif', 'system-ui'],
        mono: ['JetBrains Mono', 'ui-monospace', 'SFMono-Regular'],
      },
      colors: {
        primary: {
          50: '#eef2ff',
          500: '#4f46e5',
          600: '#4338ca',
          700: '#3730a3',
        },
      },
    },
  },
  plugins: [
    typography,
  ],
}