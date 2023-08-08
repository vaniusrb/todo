/** @type {import('tailwindcss').Config} */
module.exports = {
    content: {
        files: ["*.html", "./src/**/*.rs"],
    },
    theme: {
        extend: {},
    },
    plugins: [
        require("daisyui")
    ],
    daisyui: {
        logs: false, // Shows info about daisyUI version and used config in the console when building your CSS
    },
}
