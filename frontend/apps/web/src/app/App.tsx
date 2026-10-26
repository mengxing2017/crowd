import { Console } from '../features/community/Console'
import { Landing } from '../features/community/Landing'
export function App() { const code = new URLSearchParams(window.location.search).get('source'); return code ? <Landing code={code} /> : <Console /> }
