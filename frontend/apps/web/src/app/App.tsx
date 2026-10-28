import { Console } from '../features/workspace/Console'
import { Landing } from '../features/promotion/Landing'
export function App() { const code = new URLSearchParams(window.location.search).get('source'); return code !== null || new URLSearchParams(window.location.search).has('catalog') ? <Landing code={code} /> : <Console /> }
