import { Suspense, lazy } from 'react'
import { BrowserRouter, Routes, Route } from 'react-router-dom'
import { Layout } from '@/components/Layout'

const Home = lazy(() => import('@/pages/Home'))
const Experiments = lazy(() => import('@/pages/Experiments'))
const Lab = lazy(() => import('@/pages/Lab'))
const Glossary = lazy(() => import('@/pages/Glossary'))
const Falsification = lazy(() => import('@/pages/Falsification'))

/** GitHub Pages serves from `/mad-dog/`; local dev uses `/`. */
const basename =
  import.meta.env.BASE_URL === '/'
    ? undefined
    : import.meta.env.BASE_URL.replace(/\/$/, '')

function PageLoader() {
  return (
    <div className="flex flex-1 items-center justify-center p-12 text-sm text-muted-foreground">
      Loading…
    </div>
  )
}

export default function App() {
  return (
    <BrowserRouter basename={basename}>
      <Routes>
        <Route element={<Layout />}>
          <Route
            path="/"
            element={
              <Suspense fallback={<PageLoader />}>
                <Home />
              </Suspense>
            }
          />
          <Route
            path="/experiments"
            element={
              <Suspense fallback={<PageLoader />}>
                <Experiments />
              </Suspense>
            }
          />
          <Route
            path="/lab"
            element={
              <Suspense fallback={<PageLoader />}>
                <Lab />
              </Suspense>
            }
          />
          <Route
            path="/glossary"
            element={
              <Suspense fallback={<PageLoader />}>
                <Glossary />
              </Suspense>
            }
          />
          <Route
            path="/falsification"
            element={
              <Suspense fallback={<PageLoader />}>
                <Falsification />
              </Suspense>
            }
          />
        </Route>
      </Routes>
    </BrowserRouter>
  )
}
