import { onUnmounted } from "vue";
import { useMagicKeys, whenever } from "@vueuse/core";
import { useRouter } from "vue-router";

export function useKeyboard(options: {
  onToggleLog?: () => void;
  onToggleOpsDock?: () => void;
  onFocusSearch?: () => void;
  onEscape?: () => void;
}) {
  const router = useRouter();
  const keys = useMagicKeys();

  // Ctrl+1-4: Navigate to pages
  whenever(keys["ctrl+1"]!, () => router.push("/"));
  whenever(keys["ctrl+2"]!, () => router.push("/catalog"));
  whenever(keys["ctrl+3"]!, () => router.push("/installed"));
  whenever(keys["ctrl+4"]!, () => router.push("/settings"));

  // Ctrl+F: Focus search
  function handleKeydown(e: KeyboardEvent) {
    if (e.ctrlKey && e.key === "f") {
      e.preventDefault();
      options.onFocusSearch?.();
    }
    if (e.ctrlKey && e.key === "l") {
      e.preventDefault();
      options.onToggleLog?.();
    }
    if (e.ctrlKey && e.key === "j") {
      e.preventDefault();
      options.onToggleOpsDock?.();
    }
    if (e.ctrlKey && e.key === ",") {
      e.preventDefault();
      router.push("/settings");
    }
    if (e.key === "Escape") {
      options.onEscape?.();
    }
  }

  document.addEventListener("keydown", handleKeydown);

  onUnmounted(() => {
    document.removeEventListener("keydown", handleKeydown);
  });
}
