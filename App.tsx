import { invoke } from '@tauri-apps/api/tauri';
import { useState, useEffect } from 'react';
import './styles/index.css';

interface SteamControllerInfo {
  connected: boolean;
  connection_type: string;
  product_name: string;
  serial: string;
}

interface ButtonState {
  a: boolean;
  b: boolean;
  x: boolean;
  y: boolean;
  lb: boolean;
  rb: boolean;
  lt: boolean;
  rt: boolean;
  lgrip: boolean;
  rgrip: boolean;
  start: boolean;
  select: boolean;
  steam: boolean;
  lpad_click: boolean;
  rpad_click: boolean;
  stick_click: boolean;
}

interface TrackpadData {
  x: number;
  y: number;
  active: boolean;
}

interface StickData {
  x: number;
  y: number;
}

interface TriggersData {
  left: number;
  right: number;
}

interface GyroData {
  pitch: number;
  yaw: number;
  roll: number;
}

interface ControllerInput {
  buttons: ButtonState;
  left_trackpad: TrackpadData;
  right_trackpad: TrackpadData;
  stick: StickData;
  triggers: TriggersData;
  gyro: GyroData;
  timestamp: number;
}

function App() {
  const [controllerInfo, setControllerInfo] = useState<SteamControllerInfo | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const [error, setError] = useState<string>('');
  const [input, setInput] = useState<ControllerInput | null>(null);
  const [isPolling, setIsPolling] = useState(false);

  // Check connection status periodically
  useEffect(() => {
    const checkConnection = async () => {
      try {
        const connected = await invoke<boolean>('is_steam_controller_connected');
        setIsConnected(connected);
      } catch (e) {
        console.error('Failed to check connection:', e);
      }
    };

    checkConnection();
    const interval = setInterval(checkConnection, 1000);
    return () => clearInterval(interval);
  }, []);

  // Poll for input when connected
  useEffect(() => {
    if (!isConnected || !isPolling) return;

    const pollInput = async () => {
      try {
        const inputData = await invoke<ControllerInput>('read_controller_input');
        setInput(inputData);
        setError('');
      } catch (e) {
        // Silently ignore "No data available" errors
        const errorMsg = String(e);
        if (!errorMsg.includes('No data available')) {
          setError(errorMsg);
        }
      }
    };

    const interval = setInterval(pollInput, 16); // ~60Hz
    return () => clearInterval(interval);
  }, [isConnected, isPolling]);

  const detectController = async () => {
    try {
      const info = await invoke<SteamControllerInfo | null>('detect_steam_controller');
      setControllerInfo(info);
      setError('');
    } catch (e) {
      setError(String(e));
    }
  };

  const connectController = async () => {
    try {
      const info = await invoke<SteamControllerInfo>('connect_steam_controller');
      setControllerInfo(info);
      setIsConnected(true);
      setIsPolling(true);
      setError('');
    } catch (e) {
      setError(String(e));
    }
  };

  const disconnectController = async () => {
    try {
      await invoke('disconnect_steam_controller');
      setIsConnected(false);
      setIsPolling(false);
      setInput(null);
      setError('');
    } catch (e) {
      setError(String(e));
    }
  };

  const testRawInput = async () => {
    try {
      const rawData = await invoke<string>('read_raw_input_debug');
      console.log('✅ Raw Input Data:');
      console.log(rawData);
      setError('Raw data logged to console (F12)');
    } catch (e) {
      console.error('❌ Error reading raw input:', e);
      setError(String(e));
    }
  };

  const listInterfaces = async () => {
    try {
      const interfaces = await invoke<any[]>('list_steam_controller_interfaces');
      console.log('🔍 Steam Controller HID Interfaces:');
      console.table(interfaces);
      setError(`Found ${interfaces.length} interfaces - check console (F12)`);
    } catch (e) {
      console.error('❌ Error listing interfaces:', e);
      setError(String(e));
    }
  };

  return (
    <div className="min-h-screen bg-gray-900 text-white p-8">
      <div className="max-w-6xl mx-auto">
        <h1 className="text-4xl font-bold mb-8 text-center">
          🎮 CtrlSpace - Steam Controller Manager
        </h1>

        {/* Connection Status */}
        <div className="bg-gray-800 rounded-lg p-6 mb-6">
          <h2 className="text-2xl font-semibold mb-4">Connection Status</h2>

          <div className="flex items-center gap-4 mb-4">
            <div className={`w-4 h-4 rounded-full ${isConnected ? 'bg-green-500' : 'bg-red-500'}`} />
            <span className="text-lg">
              {isConnected ? 'Connected' : 'Not Connected'}
            </span>
          </div>

          <div className="flex gap-4">
            <button
              onClick={detectController}
              className="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded transition"
            >
              Detect Controller
            </button>
            <button
              onClick={connectController}
              disabled={isConnected}
              className={`px-4 py-2 rounded transition ${
                isConnected
                  ? 'bg-gray-600 cursor-not-allowed'
                  : 'bg-green-600 hover:bg-green-700'
              }`}
            >
              Connect
            </button>
            <button
              onClick={disconnectController}
              disabled={!isConnected}
              className={`px-4 py-2 rounded transition ${
                !isConnected
                  ? 'bg-gray-600 cursor-not-allowed'
                  : 'bg-red-600 hover:bg-red-700'
              }`}
            >
              Disconnect
            </button>
            <button
              onClick={testRawInput}
              disabled={!isConnected}
              className={`px-4 py-2 rounded transition ${
                !isConnected
                  ? 'bg-gray-600 cursor-not-allowed'
                  : 'bg-yellow-600 hover:bg-yellow-700'
              }`}
            >
              🐛 Debug Raw Input
            </button>
            <button
              onClick={listInterfaces}
              className="px-4 py-2 bg-purple-600 hover:bg-purple-700 rounded transition"
            >
              🔍 List HID Interfaces
            </button>
          </div>

          {controllerInfo && (
            <div className="mt-4 p-4 bg-gray-700 rounded">
              <p><strong>Product:</strong> {controllerInfo.product_name}</p>
              <p><strong>Connection:</strong> {controllerInfo.connection_type}</p>
              <p><strong>Serial:</strong> {controllerInfo.serial}</p>
            </div>
          )}

          {error && (
            <div className="mt-4 p-4 bg-red-900 border border-red-600 rounded">
              <p className="text-red-200">{error}</p>
            </div>
          )}
        </div>

        {/* Input Debug View */}
        {isConnected && (
          <div className="bg-gray-800 rounded-lg p-6">
            <h2 className="text-2xl font-semibold mb-4">Input Debug View</h2>

            {input ? (
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                {/* Buttons */}
                <div className="bg-gray-700 rounded p-4">
                  <h3 className="text-xl font-semibold mb-3">Buttons</h3>
                  <div className="grid grid-cols-2 gap-2 text-sm">
                    <ButtonIndicator label="A" active={input.buttons.a} />
                    <ButtonIndicator label="B" active={input.buttons.b} />
                    <ButtonIndicator label="X" active={input.buttons.x} />
                    <ButtonIndicator label="Y" active={input.buttons.y} />
                    <ButtonIndicator label="LB" active={input.buttons.lb} />
                    <ButtonIndicator label="RB" active={input.buttons.rb} />
                    <ButtonIndicator label="LT" active={input.buttons.lt} />
                    <ButtonIndicator label="RT" active={input.buttons.rt} />
                    <ButtonIndicator label="L-Grip" active={input.buttons.lgrip} />
                    <ButtonIndicator label="R-Grip" active={input.buttons.rgrip} />
                    <ButtonIndicator label="Start" active={input.buttons.start} />
                    <ButtonIndicator label="Select" active={input.buttons.select} />
                    <ButtonIndicator label="Steam" active={input.buttons.steam} />
                    <ButtonIndicator label="L-Pad Click" active={input.buttons.lpad_click} />
                    <ButtonIndicator label="R-Pad Click" active={input.buttons.rpad_click} />
                    <ButtonIndicator label="Stick Click" active={input.buttons.stick_click} />
                  </div>
                </div>

                {/* Analog Inputs */}
                <div className="bg-gray-700 rounded p-4">
                  <h3 className="text-xl font-semibold mb-3">Analog Triggers</h3>
                  <div className="space-y-2">
                    <ProgressBar label="Left Trigger" value={input.triggers.left} max={255} />
                    <ProgressBar label="Right Trigger" value={input.triggers.right} max={255} />
                  </div>
                </div>

                {/* Stick */}
                <div className="bg-gray-700 rounded p-4">
                  <h3 className="text-xl font-semibold mb-3">Analog Stick</h3>
                  <p className="text-sm">X: {input.stick.x}</p>
                  <p className="text-sm">Y: {input.stick.y}</p>
                  <div className="mt-2 w-full h-32 bg-gray-900 rounded relative">
                    <StickVisualizer x={input.stick.x} y={input.stick.y} />
                  </div>
                </div>

                {/* Left Trackpad */}
                <div className="bg-gray-700 rounded p-4">
                  <h3 className="text-xl font-semibold mb-3">Left Trackpad</h3>
                  <p className="text-sm">X: {input.left_trackpad.x}</p>
                  <p className="text-sm">Y: {input.left_trackpad.y}</p>
                  <p className="text-sm">Active: {input.left_trackpad.active ? 'Yes' : 'No'}</p>
                  <div className="mt-2 w-full h-32 bg-gray-900 rounded relative">
                    <TrackpadVisualizer data={input.left_trackpad} />
                  </div>
                </div>

                {/* Right Trackpad */}
                <div className="bg-gray-700 rounded p-4">
                  <h3 className="text-xl font-semibold mb-3">Right Trackpad</h3>
                  <p className="text-sm">X: {input.right_trackpad.x}</p>
                  <p className="text-sm">Y: {input.right_trackpad.y}</p>
                  <p className="text-sm">Active: {input.right_trackpad.active ? 'Yes' : 'No'}</p>
                  <div className="mt-2 w-full h-32 bg-gray-900 rounded relative">
                    <TrackpadVisualizer data={input.right_trackpad} />
                  </div>
                </div>

                {/* Gyro */}
                <div className="bg-gray-700 rounded p-4">
                  <h3 className="text-xl font-semibold mb-3">Gyroscope</h3>
                  <p className="text-sm">Pitch: {input.gyro.pitch}</p>
                  <p className="text-sm">Yaw: {input.gyro.yaw}</p>
                  <p className="text-sm">Roll: {input.gyro.roll}</p>
                </div>
              </div>
            ) : (
              <p className="text-gray-400">Waiting for input data...</p>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

// Helper Components
function ButtonIndicator({ label, active }: { label: string; active: boolean }) {
  return (
    <div className={`px-3 py-2 rounded text-center ${active ? 'bg-green-600' : 'bg-gray-600'}`}>
      {label}
    </div>
  );
}

function ProgressBar({ label, value, max }: { label: string; value: number; max: number }) {
  const percentage = (value / max) * 100;
  return (
    <div>
      <div className="flex justify-between text-sm mb-1">
        <span>{label}</span>
        <span>{value}</span>
      </div>
      <div className="w-full bg-gray-900 rounded h-4">
        <div
          className="bg-blue-600 h-4 rounded transition-all"
          style={{ width: `${percentage}%` }}
        />
      </div>
    </div>
  );
}

function StickVisualizer({ x, y }: { x: number; y: number }) {
  // Convert stick values (-32768 to 32767) to percentage (0-100)
  const xPercent = ((x + 32768) / 65535) * 100;
  const yPercent = ((y + 32768) / 65535) * 100;

  return (
    <div
      className="absolute w-4 h-4 bg-red-500 rounded-full transform -translate-x-1/2 -translate-y-1/2"
      style={{ left: `${xPercent}%`, top: `${100 - yPercent}%` }}
    />
  );
}

function TrackpadVisualizer({ data }: { data: TrackpadData }) {
  if (!data.active) return null;

  // Convert trackpad values (-32768 to 32767) to percentage (0-100)
  const xPercent = ((data.x + 32768) / 65535) * 100;
  const yPercent = ((data.y + 32768) / 65535) * 100;

  return (
    <div
      className="absolute w-3 h-3 bg-blue-500 rounded-full transform -translate-x-1/2 -translate-y-1/2"
      style={{ left: `${xPercent}%`, top: `${100 - yPercent}%` }}
    />
  );
}

export default App;
