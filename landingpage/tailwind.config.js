/** @type {import('tailwindcss').Config} */
export default {
    content: ["./src/**/*.astro"],
    theme: {
        colors: {
            'white': '#fff',
            'black': '#000',
            'transparent': 'transparent',
            'primary': '#c0cc19',
            'secondary': '#4a5568',
            'tertiary': 'rgb(243 244 246)',
        },
        spacing: {
            '1': '8px',
            '2': '12px',
            '3': '16px',
            '4': '24px',
            '5': '32px',
            '6': '48px',
            '16': '64px',
            '36': '144px',
            '40': '156px'
        },
        extend: {},
    },
    plugins: [],
}