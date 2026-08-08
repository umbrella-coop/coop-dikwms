import { BrowserRouter } from 'react-router-dom';
import {
  createRoot,
  type Root,
  // hydrateRoot
} from 'react-dom/client';
import { App } from "./app.js";
import "./app.module.css";

/**
 * comment this in for server-side rendering (ssr) and comment 
 * out of the root.render() invocation below.
*/
// hydrateRoot(
//   document.getElementById("root") as HTMLElement,
//   <BrowserRouter>
//     <App />
//   </BrowserRouter>
// );

if (import.meta.hot) {
  import.meta.hot.accept();
}
  
/**
 * mounting for client side rendering.
 * HMR note: vite re-evaluates this entry module on every hot update —
 * createRoot() must run once, then root.render() reuses the existing root
 * (React 18+ warns on double createRoot).
 */
const container = document.getElementById('root');
const rootRef = window as unknown as { __DIKWMS_ROOT__?: Root };
if (!rootRef.__DIKWMS_ROOT__) {
  rootRef.__DIKWMS_ROOT__ = createRoot(container!);
}
rootRef.__DIKWMS_ROOT__.render(
  <BrowserRouter>
    <App />
  </BrowserRouter>
);