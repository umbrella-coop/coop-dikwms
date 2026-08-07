import { BrowserRouter } from 'react-router-dom';
import {
  createRoot,
  // hydrateRoot
} from 'react-dom/client';
import { CoopGraphApp } from "./coop-graph-app.js";
import "./coop-graph-app.module.css";

/**
 * comment this in for server-side rendering (ssr) and comment 
 * out of the root.render() invocation below.
*/
// hydrateRoot(
//   document.getElementById("root") as HTMLElement,
//   <BrowserRouter>
//     <CoopGraphApp />
//   </BrowserRouter>
// );

if (import.meta.hot) {
  import.meta.hot.accept();
}
  
/**
 * mounting for client side rendering.
 */
const container = document.getElementById('root');
const root = createRoot(container!);

root.render(
  <BrowserRouter>
    <CoopGraphApp />
  </BrowserRouter>
);