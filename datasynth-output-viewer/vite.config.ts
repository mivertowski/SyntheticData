import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
          server  : {
            allowedHosts :['6fe6-90-102-79-161.ngrok-free.app']
          }
})
