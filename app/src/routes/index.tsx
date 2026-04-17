import { createFileRoute } from '@tanstack/react-router'
import { TradingView } from '../components/TradingView'

export const Route = createFileRoute('/')({ component: App })

function App() {
  return <TradingView />
}
