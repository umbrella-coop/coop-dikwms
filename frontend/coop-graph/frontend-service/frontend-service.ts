import { mockAnnouncements } from "@grps/coop-graph.entities.announcement";

/**
 * corporate service
 */
export class FrontendService {
  /**
   * say hello.
   */
  async listAnnouncements() {
    return mockAnnouncements();
  }

  /**
   * create a new instance of a corporate service.
   */
  static from() {
    return new FrontendService();
  }
}    
