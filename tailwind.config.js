/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        arkive: {
          primary: '#3b82f6',
          secondary: '#1e293b',
          accent: '#8b5cf6',
        }
      }
    },
  },
  plugins: [],
}
