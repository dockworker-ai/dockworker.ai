import { create } from 'zustand';

interface User {
  id: string;
  username: string;
  email: string;
  tier: 'free' | 'pro' | 'team' | 'enterprise';
  token: string;
}

interface Build {
  id: string;
  status: 'QUEUED' | 'RUNNING' | 'SUCCEEDED' | 'FAILED' | 'TIMEOUT' | 'CANCELLED';
  repoUrl: string;
  imageName: string;
  createdAt: string;
  startedAt?: string;
  completedAt?: string;
  duration?: number;
}

interface Store {
  user: User | null;
  builds: Build[];
  quota: {
    monthlyMinutes: number;
    remainingMinutes: number;
    maxConcurrent: number;
    resetDate: string;
  };
  setUser: (user: User | null) => void;
  setBuilds: (builds: Build[]) => void;
  setQuota: (quota: any) => void;
  logout: () => void;
}

export const useStore = create<Store>((set) => ({
  user: null,
  builds: [],
  quota: {
    monthlyMinutes: 200,
    remainingMinutes: 200,
    maxConcurrent: 1,
    resetDate: new Date().toISOString(),
  },
  setUser: (user) => set({ user }),
  setBuilds: (builds) => set({ builds }),
  setQuota: (quota) => set({ quota }),
  logout: () => set({ user: null, builds: [] }),
}));
