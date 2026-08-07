import { MockProvider } from '@acme/acme.testing.mock-provider';
import { FrontendWeb } from "./frontend-web.js";
    
export const FrontendWebBasic = () => {
  return (
    <MockProvider noTheme>
      <FrontendWeb />
    </MockProvider>
  );
}
