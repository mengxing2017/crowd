import { Console } from '../features/community/Console'
import { Landing } from '../features/community/Landing'
export function App() { const code = new URLSearchParams(window.location.search).get('source'); return code !== null || new URLSearchParams(window.location.search).has('catalog') ? <Landing code={code} /> : <Console /> }
