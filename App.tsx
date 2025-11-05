import { invoke } from '@tauri-apps/api/tauri'
import { useState } from 'react'
import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import './styles/index.css'



ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)
function App() {
  const [greetMsg, setGreetMsg] = useState('')
  const [name, setName] = useState('')

  async function greet() {
    setGreetMsg(await invoke('greet', { name }))
  }

  return (
    <div className="container">
      <h1>Welcome to CtrlSpace!</h1>
      <div className="row">
        <input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button type="button" onClick={() => greet()}>
          Greet
        </button>
      </div>
      <p>{greetMsg}</p>
    </div>
  )
}

export default App