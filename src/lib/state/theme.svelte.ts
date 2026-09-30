export type ThemeMode = 'dark' | 'light' | 'auto';

class ThemeStore {
  mode = $state<ThemeMode>('auto');
  resolvedTheme = $state<'dark' | 'light'>('dark');

  constructor() {
    if (typeof window !== 'undefined') {
      try {
        const saved = localStorage.getItem('fugdb_theme_mode') as ThemeMode | null;
        if (saved && (saved === 'dark' || saved === 'light' || saved === 'auto')) {
          this.mode = saved;
        }
      } catch {}

      this.updateResolvedTheme();

      // Listen to OS system theme changes
      try {
        const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
        mediaQuery.addEventListener('change', () => {
          if (this.mode === 'auto') {
            this.updateResolvedTheme();
          }
        });
      } catch {}
    }
  }

  setMode(newMode: ThemeMode) {
    this.mode = newMode;
    try {
      localStorage.setItem('fugdb_theme_mode', newMode);
    } catch {}
    this.updateResolvedTheme();
  }

  toggleCycle() {
    if (this.mode === 'auto') {
      this.setMode('dark');
    } else if (this.mode === 'dark') {
      this.setMode('light');
    } else {
      this.setMode('auto');
    }
  }

  private updateResolvedTheme() {
    if (typeof window === 'undefined') return;

    let isDark = true;
    if (this.mode === 'dark') {
      isDark = true;
    } else if (this.mode === 'light') {
      isDark = false;
    } else {
      // auto
      isDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    }

    this.resolvedTheme = isDark ? 'dark' : 'light';

    if (typeof document !== 'undefined') {
      const root = document.documentElement;
      if (isDark) {
        root.classList.add('dark');
        root.classList.remove('light');
        root.style.colorScheme = 'dark';
      } else {
        root.classList.remove('dark');
        root.classList.add('light');
        root.style.colorScheme = 'light';
      }
    }
  }
}

export const themeStore = new ThemeStore();
