import React from 'react';
import { Button, Table, Typography } from 'antd';
import { PlusOutlined } from '@ant-design/icons';

const { Title } = Typography;

const Projects: React.FC = () => {
  return (
    <div>
      <Title level={2}>Gestion des Projets</Title>
      <Button type="primary" icon={<PlusOutlined />}>
        Nouveau Projet
      </Button>
      <Table
        dataSource={[]}
        columns={[
          { title: 'Nom', dataIndex: 'name', key: 'name' },
          { title: 'Langage', dataIndex: 'language', key: 'language' },
          { title: 'Chemin', dataIndex: 'path', key: 'path' },
        ]}
        style={{ marginTop: 16 }}
      />
    </div>
  );
};

export default Projects;
