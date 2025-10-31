import { create } from 'zustand'

interface AppState {
  profiles: any[]
  currentProfile: any
  setProfiles: (profiles: any[]) => void
  setCurrentProfile: (profile: any) => void
}

export const useStore = create<AppState>((set) => ({
  profiles: [],
  currentProfile: null,
  setProfiles: (profiles) => set({ profiles }),
  setCurrentProfile: (profile) => set({ currentProfile: profile }),
}))