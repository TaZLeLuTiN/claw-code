import React, { useState } from 'react';
import { Layout, Menu } from 'antd';
import { 
  DashboardOutlined, 
  ProjectOutlined, 
  SettingOutlined,
  MessageOutlined  // ← MessageOutlined importé avec les autres
} from '@ant-design/icons';
import Dashboard from './pages/Dashboard';
import Projects from './pages/Projects';
import Settings from './pages/Settings';
import Chat from './pages/Chat';

const { Header, Sider, Content } = Layout;

const App: React.FC = () => {
  const [selectedKey, setSelectedKey] = useState('dashboard');

  const renderContent = () => {
    switch (selectedKey) {
      case 'dashboard':
        return <Dashboard />;
      case 'projects':
        return <Projects />;
      case 'chat':  // ← Cas pour le chat
        return <Chat />;
      case 'settings':
        return <Settings />;
      default:
        return <Dashboard />;
    }
  };

  return (
    <Layout style={{ minHeight: '100vh' }}>
      <Sider collapsible>
        <div style={{ 
          height: 32, 
          margin: 16, 
          background: 'rgba(255, 255, 255, 0.2)',
          borderRadius: 6,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          color: 'white',
          fontWeight: 'bold'
        }}>
          CLAW UI
        </div>
        <Menu
          theme="dark"
          selectedKeys={[selectedKey]}
          mode="inline"
          onSelect={({ key }) => setSelectedKey(key as string)}
          items={[
            {
              key: 'chat',
              icon: <MessageOutlined />,
              label: 'Chat IA',
            },  // ← VIRGULE AJOUTÉE ICI
            {
              key: 'dashboard',
              icon: <DashboardOutlined />,
              label: 'Dashboard',
            },
            {
              key: 'projects',
              icon: <ProjectOutlined />,
              label: 'Projets',
            },
            {
              key: 'settings',
              icon: <SettingOutlined />,
              label: 'Paramètres',
            },
          ]}
        />
      </Sider>
      <Layout>
        <Header style={{ 
          padding: '0 24px', 
          background: '#fff',
          boxShadow: '0 1px 4px rgba(0,21,41,.08)',
          display: 'flex',
          alignItems: 'center'
        }}>
          <h2>CLAW Framework - Interface de Gestion</h2>
        </Header>
        <Content style={{ 
          margin: '24px 16px', 
          padding: 24, 
          background: '#fff',
          minHeight: 280
        }}>
          {renderContent()}
        </Content>
      </Layout>
    </Layout>
  );
};

export default App;
