/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        bg: '#0A0E17',
        surface: '#161F30',
        borderSubtle: '#2A364F',
        primary: '#00E5FF',
        critical: '#FF2A5F',
        warning: '#FFB800',
        secure: '#10B981',
        textMain: '#FFFFFF',
        textMuted: '#9CA3AF'
      },
      fontFamily: {
        sans: ['Inter', 'sans-serif'],
      },
      boxShadow: {
        'glow': '0 0 15px rgba(0, 229, 255, 0.3)',
        'glow-critical': '0 0 15px rgba(255, 42, 95, 0.3)',
      },
      animation: {
        'pulse-slow': 'pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
      }
    },
  },
  plugins: [],
}
