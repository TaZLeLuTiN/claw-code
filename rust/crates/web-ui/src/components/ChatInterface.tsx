import React, { useState } from 'react';
import { Card, Input, Button, List, Typography, Select, Space } from 'antd';
import { SendOutlined, RobotOutlined } from '@ant-design/icons';
//import type { AIModel } from '../types';
import FormattedResponse from './FormattedResponse';

const { TextArea } = Input;
const { Option } = Select;
const { Title } = Typography;

interface Message {
  role: 'user' | 'assistant';
  content: string;
  timestamp: Date;
  model?: string;
}

const ChatInterface: React.FC = () => {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState('');
  const [selectedModel, setSelectedModel] = useState('mistral:7b');
  const [isLoading, setIsLoading] = useState(false);
  const sendMessage = async () => {
    if (!input.trim()) return;
    
    const userMessage: Message = {
      role: 'user',
      content: input,
      timestamp: new Date()
    };
    
    setMessages(prev => [...prev, userMessage]);
    setIsLoading(true);
    
    try {
      const response = await fetch('/api/ollama/generate', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ model: selectedModel, prompt: input })
      });
      
      const data = await response.json();
      
      const assistantMessage: Message = {
        role: 'assistant',
        content: data.text,
        timestamp: new Date(),
        model: selectedModel
      };
      
      setMessages(prev => [...prev, assistantMessage]);
    } catch (error) {
      console.error('Error:', error);
    } finally {
      setIsLoading(false);
      setInput('');
    }
  };

  return (
    <Card>
      <Title level={3}>💬 Chat IA BMAD</Title>
      
      <Space direction="vertical" style={{ width: '100%' }} size="middle">
        <Select 
          value={selectedModel} 
          onChange={setSelectedModel}
          style={{ width: 200 }}
        >
          <Option value="mistral:7b">Mistral 7B</Option>
          <Option value="codellama:7b">CodeLlama 7B</Option>
          <Option value="deepseek-coder-v2:latest">DeepSeek Coder V2</Option>
          <Option value="llama3.1:70b-instruct-q4_K_M">Llama 3.1 70B</Option>
        </Select>

        <Card style={{ height: 400, overflow: 'auto' }}>
          <List
            dataSource={messages}
            renderItem={(msg) => (
              <List.Item>
                <Card 
                  size="small" 
                  style={{ 
                    textAlign: msg.role === 'user' ? 'right' : 'left',
                    background: msg.role === 'assistant' ? '#f0f8ff' : '#fff'
                  }}
                >
                  <strong>{msg.role === 'user' ? '👤 Vous' : '🤖 IA'}:</strong>
                  <br />
                  <FormattedResponse content={msg.content} />
                  <br />
                  <small style={{ color: '#666' }}>
                    {msg.timestamp.toLocaleTimeString()}
                    {msg.model && ` • ${msg.model}`}
                  </small>
                </Card>
              </List.Item>
            )}
          />
          {isLoading && (
            <div style={{ textAlign: 'center', padding: 10 }}>
              <RobotOutlined spin /> Génération en cours...
            </div>
          )}
        </Card>

        <Space.Compact style={{ width: '100%' }}>
          <TextArea
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder="Posez votre question..."
            rows={3}
            onPressEnter={(e) => {
              if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                sendMessage();
              }
            }}
          />
          <Button 
            type="primary" 
            onClick={sendMessage}
            loading={isLoading}
            icon={<SendOutlined />}
          >
            Envoyer
          </Button>
        </Space.Compact>
      </Space>
    </Card>
  );
};

export default ChatInterface;

