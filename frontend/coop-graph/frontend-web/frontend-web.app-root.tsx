import { BrowserRouter } from 'react-router-dom';
import { 
  createRoot, 
  // hydrateRoot 
} from 'react-dom/client';
import { AcmeApolloProvider } from './apollo-provider.js';
import { FrontendWeb } from "./frontend-web.js";

if (import.meta.hot) {
  import.meta.hot.accept();
}

/**
 * comment this in for server-side rendering (ssr) and comment 
 * out of the root.render() invocation below.
*/
// hydrateRoot(
//   document.getElementById("root") as HTMLElement,
//   <BrowserRouter>
//     <FrontendWeb />
//   </BrowserRouter>
// );

/**
 * mounting for client side rendering.
 */
const container = document.getElementById('root');
const root = createRoot(container!);

root.render(
  <BrowserRouter>
    <AcmeApolloProvider>
      <FrontendWeb />
    </AcmeApolloProvider>
  </BrowserRouter>
);
