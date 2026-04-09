import React from 'react';
import { Card, Typography } from 'antd';

const { Title } = Typography;

const Settings: React.FC = () => {
  return (
    <div>
      <Title level={2}>Paramètres</Title>
      <Card title="Configuration">
        <p>Paramètres de l'application à venir...</p>
      </Card>
    </div>
  );
};

export default Settings;
