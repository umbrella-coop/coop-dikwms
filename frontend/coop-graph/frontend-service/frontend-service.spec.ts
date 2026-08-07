import { FrontendService } from './frontend-service.js';

describe('corporate service', () => {
  it('should say hello', async () => {
    const frontendService = FrontendService.from();
    const announcements = await frontendService.listAnnouncements();
    expect(announcements.length).toEqual(2);
  })
});
