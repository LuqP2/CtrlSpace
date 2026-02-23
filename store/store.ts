import { create } from "zustand";

export interface Profile {
  id: string;
  name: string;
  map_a_to_space: boolean;
}

interface AppState {
  profiles: Profile[];
  currentProfile: Profile | null;
  setProfiles: (profiles: Profile[]) => void;
  setCurrentProfile: (profile: Profile | null) => void;
}

export const useStore = create<AppState>((set) => ({
  profiles: [{ id: "1", name: "Default Mapper Setup", map_a_to_space: true }],
  currentProfile: {
    id: "1",
    name: "Default Mapper Setup",
    map_a_to_space: true,
  },
  setProfiles: (profiles) => set({ profiles }),
  setCurrentProfile: (profile) => set({ currentProfile: profile }),
}));
