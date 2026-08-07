import { ReactNode } from 'react';
import { ApolloClient, InMemoryCache, HttpLink } from '@apollo/client';
import { ApolloProvider } from '@apollo/client/react';

export type BeerShopApolloProviderProps = {
  children: ReactNode;
};

export function AcmeApolloProvider({ children }: BeerShopApolloProviderProps) {
  const gatewayUrl = process.env?.BACKEND_URL;
  
  const client = new ApolloClient({
    link: new HttpLink({
      uri: gatewayUrl,
      credentials: 'same-origin',
    }),
    cache: new InMemoryCache(),
  });
  
  return (
    <ApolloProvider client={client}>
      {children}
    </ApolloProvider>
  )
}
