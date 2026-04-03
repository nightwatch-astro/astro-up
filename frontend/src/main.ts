import { createApp } from "vue";
import PrimeVue from "primevue/config";
import Aura from "@primeuix/themes/aura";
import ToastService from "primevue/toastservice";
import { VueQueryPlugin } from "@tanstack/vue-query";
import router from "./router";
import App from "./App.vue";
import "./styles.css";

const app = createApp(App);

app.use(router);
app.use(PrimeVue, {
  theme: {
    preset: Aura,
    options: {
      darkModeSelector: ".app-dark",
      cssLayer: false,
    },
  },
});

app.use(ToastService);
app.use(VueQueryPlugin);

app.mount("#app");
