import { ref, computed, type Ref } from "vue";
import type { PackageSummary, Category } from "../types/package";

export function useSearch<T extends PackageSummary>(items: Ref<T[] | undefined>) {
  const searchText = ref("");
  const activeCategory = ref<Category | null>(null);

  const filtered = computed(() => {
    if (!items.value) return [];

    let result = items.value;

    // Category filter
    if (activeCategory.value) {
      result = result.filter((p) => p.category === activeCategory.value);
    }

    // Text search with relevance ranking (FR-053)
    const query = searchText.value.trim().toLowerCase();
    if (!query) {
      // Browse mode: sort alphabetically by name
      return [...result].sort((a, b) => a.name.localeCompare(b.name));
    }

    return result
      .map((p) => {
        let rank = 0;
        const id = p.id.toLowerCase();
        const name = p.name.toLowerCase();
        const desc = (p.description ?? "").toLowerCase();
        const tags = p.tags.map((t) => t.toLowerCase());
        const aliases = p.aliases.map((a) => a.toLowerCase());
        const deps = p.dependencies.map((d) => d.toLowerCase());
        const license = (p.license ?? "").toLowerCase();

        // Exact ID match = highest (FR-053)
        if (id === query) rank = 110;
        // Exact name match
        else if (name === query) rank = 100;
        // Name starts with query
        else if (name.startsWith(query)) rank = 80;
        // ID contains query
        else if (id.includes(query)) rank = 70;
        // Name contains query
        else if (name.includes(query)) rank = 60;
        // Alias match
        else if (aliases.some((a) => a.includes(query))) rank = 50;
        // Tag match
        else if (tags.some((t) => t.includes(query))) rank = 40;
        // Publisher match
        else if (p.publisher?.toLowerCase().includes(query)) rank = 30;
        // License match
        else if (license.includes(query)) rank = 25;
        // Dependencies match
        else if (deps.some((d) => d.includes(query))) rank = 22;
        // Description match
        else if (desc.includes(query)) rank = 20;

        return { item: p, rank };
      })
      .filter((r) => r.rank > 0)
      .sort((a, b) => b.rank - a.rank)
      .map((r) => r.item);
  });

  function setCategory(category: Category | null) {
    activeCategory.value = category;
  }

  function clearSearch() {
    searchText.value = "";
    activeCategory.value = null;
  }

  return {
    searchText,
    activeCategory,
    filtered,
    setCategory,
    clearSearch,
  };
}
