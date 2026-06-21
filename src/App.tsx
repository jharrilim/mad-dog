import { HashRouter, Routes, Route } from 'react-router-dom'
import { Layout } from '@/components/Layout'
import Home from '@/pages/Home'
import Lab from '@/pages/Lab'

export default function App() {
  return (
    <HashRouter>
      <Routes>
        <Route element={<Layout />}>
          <Route path="/" element={<Home />} />
          <Route path="/lab" element={<Lab />} />
        </Route>
      </Routes>
    </HashRouter>
  )
}
