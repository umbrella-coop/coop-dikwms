import { MemoryRouter } from 'react-router-dom';
import { CoopGraphApp } from "./coop-graph-app.js";
    
export const CoopGraphAppBasic = () => {
  return (
    <MemoryRouter>
      <CoopGraphApp />
    </MemoryRouter>
  );
}