import { createApp } from "vue";
import PrimeVue from "primevue/config";
import { definePreset } from "@primeuix/themes";
import Aura from "@primeuix/themes/aura";
import ToastService from "primevue/toastservice";
import { VueQueryPlugin } from "@tanstack/vue-query";
import router from "./router";
import App from "./App.vue";
import "./styles.css";

const AstroUpTheme = definePreset(Aura, {
  semantic: {
    primary: {
      50: "{blue.50}",
      100: "{blue.100}",
      200: "{blue.200}",
      300: "{blue.300}",
      400: "{blue.400}",
      500: "{blue.500}",
      600: "{blue.600}",
      700: "{blue.700}",
      800: "{blue.800}",
      900: "{blue.900}",
      950: "{blue.950}",
    },
    colorScheme: {
      light: {
        surface: {
          0: "#ffffff",
          50: "{stone.50}",
          100: "{stone.100}",
          200: "{stone.200}",
          300: "{stone.300}",
          400: "{stone.400}",
          500: "{stone.500}",
          600: "{stone.600}",
          700: "{stone.700}",
          800: "{stone.800}",
          900: "{stone.900}",
          950: "{stone.950}",
        },
      },
      dark: {
        surface: {
          0: "#ffffff",
          50: "{zinc.50}",
          100: "{zinc.100}",
          200: "{zinc.200}",
          300: "{zinc.300}",
          400: "{zinc.400}",
          500: "{zinc.500}",
          600: "{zinc.600}",
          700: "{zinc.700}",
          800: "{zinc.800}",
          900: "{zinc.900}",
          950: "{zinc.950}",
        },
      },
    },
  },
});

const app = createApp(App);

app.use(router);
app.use(PrimeVue, {
  theme: {
    preset: AstroUpTheme,
    options: {
      darkModeSelector: ".app-dark",
      cssLayer: false,
    },
  },
});

app.use(ToastService);
app.use(VueQueryPlugin);

// Global uncaught error handler (safety net for errors not caught by onErrorCaptured)
app.config.errorHandler = (err, instance, info) => {
  const message = err instanceof Error ? err.message : String(err);
  const component = instance?.$options?.name ?? "unknown";
  console.error(`[global error handler] ${component}: ${message} (${info})`);
};

app.mount("#app");
