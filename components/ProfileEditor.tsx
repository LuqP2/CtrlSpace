import { useStore } from '../store/store';

export function ProfileEditor() {
  const currentProfile = useStore(state => state.currentProfile);
  const setCurrentProfile = useStore(state => state.setCurrentProfile);

  if (!currentProfile) return null;

  return (
    <div className="bg-gray-800 rounded-lg p-6 mb-6 mt-6">
      <h2 className="text-2xl font-semibold mb-4">Profile: {currentProfile.name}</h2>
      
      <div className="bg-gray-700 p-4 rounded-lg">
        <h3 className="text-xl mb-3">Input Mappings (MVP)</h3>
        
        <div className="flex items-center gap-3 bg-gray-600 p-3 rounded">
          <div className="font-bold text-lg w-24">Button A</div>
          <div>➜</div>
          <div className="flex-1">
            <select 
              className="bg-gray-800 text-white rounded px-3 py-2 w-full outline-none"
              value={currentProfile.map_a_to_space ? "space" : "none"}
              onChange={(e) => {
                setCurrentProfile({
                  ...currentProfile,
                  map_a_to_space: e.target.value === "space"
                });
              }}
            >
              <option value="none">Unmapped</option>
              <option value="space">Keyboard: Spacebar</option>
            </select>
          </div>
        </div>
        
        <p className="text-xs text-gray-400 mt-4">
          * This is the MVP Mapper. Changing this dropdown updates the active profile in the app globally.
        </p>
      </div>
    </div>
  );
}
