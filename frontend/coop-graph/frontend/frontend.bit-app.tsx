import { Platform } from '@bitdev/platforms.platform';

const FrontendWeb = import.meta.resolve('@grps/coop-graph.frontend-web');
const FrontendService = import.meta.resolve('@grps/coop-graph.frontend-service');

export const Frontend = Platform.from({
  name: 'frontend',

  frontends: {
    main: FrontendWeb,
    mainPortRange: [3000, 3100]
  },

  backends: {
    main: FrontendService,
  },
});

export default Frontend;
