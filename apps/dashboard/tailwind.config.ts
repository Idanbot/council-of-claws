import type { Config } from 'tailwindcss';

const config: Config = {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  theme: {
    extend: {}
  },
  darkMode: ['class', '[data-theme="dark"]'],
  plugins: []
};

export default config;
