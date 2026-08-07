import { gql } from '@apollo/client';
import { useQuery } from '@apollo/client/react';
import { useIsMock } from '@acme/acme.testing.mock-provider';
import { Announcement, mockAnnouncements, type PlainAnnouncement } from '@grps/coop-graph.entities.announcement';

// define a graphql query
export const LIST_ANNOUNCEMENTS = gql`
  query LIST_ANNOUNCEMENTS {
    listAnnouncements {
      title
      date
    }
  }
`;

/**
 * fetch list of announcements.
 */
export function useAnnouncements(): undefined|null|Announcement[] {
  const results = useQuery<{ listAnnouncements: PlainAnnouncement[] }>(LIST_ANNOUNCEMENTS);
  const isMock = useIsMock();
  if (isMock) return mockAnnouncements();

  if (!results.data || results.loading) return undefined;
  if (!results?.data?.listAnnouncements) return null;

  return results.data.listAnnouncements.map((announcement) => {
    return Announcement.from(announcement);
  });
}
