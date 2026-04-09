import React from 'react';
import { Typography } from 'antd';
import ChatInterface from '../components/ChatInterface';

const { Title } = Typography;

const Chat: React.FC = () => {
  return (
    <div>
      <Title level={2}>💬 Chat Intelligent BMAD</Title>
      <p>Interface de conversation avec les modèles IA orchestrés par le framework BMAD</p>
      <ChatInterface />
    </div>
  );
};

export default Chat;

