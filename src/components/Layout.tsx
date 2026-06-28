import { Link, Outlet, useLocation } from 'react-router-dom'
import { cn } from '@/lib/utils'

const GITHUB_REPO_URL = 'https://github.com/jharrilim/mad-dog'

export function Layout() {
  const { pathname } = useLocation()
  const isEssay = pathname === '/'
  const isExperiments = pathname.startsWith('/experiments')
  const isLab = pathname.startsWith('/lab')
  const isGlossary = pathname.startsWith('/glossary')
  const isFalsification = pathname.startsWith('/falsification')

  return (
    <div className="min-h-svh flex flex-col">
      <nav className="border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/80 sticky top-0 z-50">
        <div className="mx-auto max-w-6xl px-6 h-12 flex items-center justify-between">
          <Link
            to="/"
            className="text-sm font-medium tracking-tight hover:text-primary transition-colors"
          >
            Mad-Dog Everettianism
          </Link>
          <div className="flex gap-1 text-sm">
            <Link
              to="/"
              className={cn(
                'px-3 py-1.5 rounded-md transition-colors',
                isEssay
                  ? 'bg-secondary text-secondary-foreground'
                  : 'text-muted-foreground hover:text-foreground',
              )}
            >
              Essay
            </Link>
            <Link
              to="/experiments"
              className={cn(
                'px-3 py-1.5 rounded-md transition-colors',
                isExperiments
                  ? 'bg-secondary text-secondary-foreground'
                  : 'text-muted-foreground hover:text-foreground',
              )}
            >
              Experiments
            </Link>
            <Link
              to="/lab"
              className={cn(
                'px-3 py-1.5 rounded-md transition-colors',
                isLab
                  ? 'bg-secondary text-secondary-foreground'
                  : 'text-muted-foreground hover:text-foreground',
              )}
            >
              Universe Lab
            </Link>
            <Link
              to="/falsification"
              className={cn(
                'px-3 py-1.5 rounded-md transition-colors',
                isFalsification
                  ? 'bg-secondary text-secondary-foreground'
                  : 'text-muted-foreground hover:text-foreground',
              )}
            >
              Falsification
            </Link>
            <Link
              to="/glossary"
              className={cn(
                'px-3 py-1.5 rounded-md transition-colors',
                isGlossary
                  ? 'bg-secondary text-secondary-foreground'
                  : 'text-muted-foreground hover:text-foreground',
              )}
            >
              Glossary
            </Link>
            <a
              href={GITHUB_REPO_URL}
              target="_blank"
              rel="noopener noreferrer"
              className="px-3 py-1.5 rounded-md transition-colors text-muted-foreground hover:text-foreground"
            >
              GitHub
            </a>
          </div>
        </div>
      </nav>
      <Outlet />
    </div>
  )
}
