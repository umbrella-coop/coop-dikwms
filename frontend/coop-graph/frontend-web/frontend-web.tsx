import { Routes, Route } from 'react-router-dom';
import { NavigationProvider, Link } from '@bitdesign/sparks.navigation.link';
import { Header } from '@bitdesign/sparks.layout.header';
import { Logo } from '@bitdesign/sparks.content.logo';
import { AcmeTheme } from '@acme/design.acme-theme';
import { Announcements } from '@grps/coop-graph.ui.announcements';

export function FrontendWeb() {
  return (
    <AcmeTheme>
      <NavigationProvider>
        <Header logo={<Logo src='https://static.bit.dev/extensions-icons/acme.svg' name='Acme' slogan='Inc.' />}>
          <Link href='/'>Investors</Link>          
          <Link href='/'>Onboarding</Link>          
        </Header>
        <Routes>
          <Route path="/" element={<Announcements />} />
        </Routes>
      </NavigationProvider>
    </AcmeTheme>
  );
}
