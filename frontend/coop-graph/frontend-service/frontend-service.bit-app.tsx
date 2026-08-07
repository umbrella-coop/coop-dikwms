import { NodeServer } from '@bitdev/node.node-server';

export default NodeServer.from({
  name: 'frontend-service',
  mainPath: './frontend-service.app-root.js',
});
